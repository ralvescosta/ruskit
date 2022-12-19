use crate::{
    client::Amqp,
    topology::{ConsumerHandler, QueueDefinition},
    types::{new_span, Metadata},
};
use errors::amqp::AmqpError;
use futures_util::{future::join_all, StreamExt};
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicNackOptions, BasicPublishOptions},
};
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{FutureExt, Span, Status},
};
use std::{borrow::Cow, sync::Arc, vec};
use tokio::task::JoinError;
use tracing::{debug, error, warn};

pub struct Dispatches {
    pub queues: Vec<QueueDefinition>,
    pub msgs_types: Vec<String>,
    pub handlers: Vec<Arc<dyn ConsumerHandler + Send + Sync>>,
}

impl Dispatches {
    pub fn new() -> Dispatches {
        Dispatches {
            queues: vec![],
            msgs_types: vec![],
            handlers: vec![],
        }
    }

    pub fn declare(
        &mut self,
        queue: QueueDefinition,
        msg_type: String,
        handler: Arc<dyn ConsumerHandler + Send + Sync>,
    ) -> Result<(), AmqpError> {
        if msg_type.is_empty() {
            return Err(AmqpError::ConsumerDeclarationError {});
        }

        self.queues.push(queue);
        self.msgs_types.push(msg_type);
        self.handlers.push(handler);

        Ok(())
    }
}

pub async fn consume_blocking(
    dispatches: &Dispatches,
    amqp: Arc<dyn Amqp + Send + Sync>,
) -> Vec<Result<(), JoinError>> {
    let mut spawns = vec![];

    for i in 0..dispatches.queues.len() {
        spawns.push(tokio::spawn({
            let m_amqp = amqp.clone();
            let msgs_allowed = dispatches.msgs_types.clone();

            let queue = dispatches.queues[i].clone();
            let msg_type = dispatches.msgs_types[i].clone();
            let handler = dispatches.handlers[i].clone();

            let mut consumer = m_amqp
                .consumer(&queue.name, &msg_type)
                .await
                .expect("unexpected error while creating the consumer");

            async move {
                while let Some(result) = consumer.next().await {
                    match result {
                        Ok(delivery) => match consume(
                            &global::tracer("amqp consumer"),
                            &queue,
                            &msg_type,
                            &msgs_allowed,
                            &delivery,
                            m_amqp.clone(),
                            handler.clone(),
                        )
                        .await
                        {
                            Err(e) => error!(error = e.to_string(), "errors consume msg"),
                            _ => {}
                        },
                        Err(e) => error!(error = e.to_string(), "error receiving delivery msg"),
                    };
                }
            }
        }));
    }

    join_all(spawns).await
}

async fn consume<'c>(
    tracer: &'c BoxedTracer,
    queue: &'c QueueDefinition,
    msg_type: &'c str,
    msgs_allowed: &'c [String],
    delivery: &'c Delivery,
    amqp: Arc<dyn Amqp + Send + Sync>,
    handler: Arc<dyn ConsumerHandler + Send + Sync>,
) -> Result<(), AmqpError> {
    let metadata = Metadata::extract(&delivery.properties);

    let (ctx, mut span) = new_span(&delivery.properties, &tracer, &metadata.msg_type);

    debug!(
        trace.id = traces::trace_id(&ctx),
        span.id = traces::span_id(&ctx),
        "received: {} - queue: {}",
        metadata.msg_type,
        queue.name,
    );

    if metadata.msg_type.is_empty() || metadata.msg_type != msg_type.to_string() {
        let msg = "unexpected or empty type - removing message";
        span.record_error(&AmqpError::ConsumerError(msg.to_string()));
        debug!(
            trace.id = traces::trace_id(&ctx),
            span.id = traces::span_id(&ctx),
            "{}",
            msg
        );
        match delivery.ack(BasicAckOptions { multiple: false }).await {
            Ok(_) => {}
            Err(e) => {
                error!("error whiling nack msg");
                span.record_error(&e);
            }
        };
        return Ok(());
    }

    if !msgs_allowed.contains(&metadata.msg_type) {
        let msg = "remove message - reason: unsupported msg type";
        span.record_error(&AmqpError::ConsumerError(msg.to_string()));
        debug!(
            trace.id = traces::trace_id(&ctx),
            span.id = traces::span_id(&ctx),
            "{}",
            msg
        );
        match delivery.ack(BasicAckOptions { multiple: false }).await {
            Ok(_) => {}
            Err(e) => {
                error!("error whiling nack msg");
                span.record_error(&e);
            }
        };
        return Ok(());
    }

    match handler
        .exec(&ctx, delivery.data.as_slice())
        .with_context(ctx.clone())
        .await
    {
        Ok(_) => match delivery.ack(BasicAckOptions { multiple: false }).await {
            Ok(_) => {
                span.set_status(Status::Ok);
                return Ok(());
            }
            Err(e) => {
                error!(
                    trace.id = traces::trace_id(&ctx),
                    span.id = traces::span_id(&ctx),
                    "error whiling ack msg"
                );
                span.record_error(&e);
                span.set_status(Status::Error {
                    description: Cow::from("error to ack msg"),
                });
                return Err(AmqpError::AckMessageError {});
            }
        },
        _ if queue.with_retry => {
            if metadata.count < queue.retries.unwrap() {
                warn!(
                    trace.id = traces::trace_id(&ctx),
                    span.id = traces::span_id(&ctx),
                    "error whiling handling msg, requeuing for latter"
                );
                match delivery
                    .nack(BasicNackOptions {
                        multiple: false,
                        requeue: false,
                    })
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        error!(
                            trace.id = traces::trace_id(&ctx),
                            span.id = traces::span_id(&ctx),
                            "error whiling requeuing"
                        );
                        span.record_error(&e);
                        span.set_status(Status::Error {
                            description: Cow::from("error to requeuing msg"),
                        });
                        return Err(AmqpError::RequeuingMessageError {});
                    }
                }
            } else {
                error!(
                    trace.id = traces::trace_id(&ctx),
                    span.id = traces::span_id(&ctx),
                    "too many attempts, sending to dlq"
                );
                match amqp
                    .channel()
                    .basic_publish(
                        "",
                        &queue.dlq_name,
                        BasicPublishOptions::default(),
                        &delivery.data,
                        delivery.properties.clone(),
                    )
                    .await
                {
                    Ok(_) => {
                        match delivery.ack(BasicAckOptions { multiple: false }).await {
                            Ok(_) => return Ok(()),
                            Err(e) => {
                                error!(
                                    trace.id = traces::trace_id(&ctx),
                                    span.id = traces::span_id(&ctx),
                                    "error whiling ack msg to default queue"
                                );
                                span.record_error(&e);
                                span.set_status(Status::Error {
                                    description: Cow::from("msg was sent to dlq"),
                                });
                                return Err(AmqpError::AckMessageError {});
                            }
                        };
                    }
                    Err(e) => {
                        error!(
                            trace.id = traces::trace_id(&ctx),
                            span.id = traces::span_id(&ctx),
                            "error whiling sending to dlq"
                        );
                        span.record_error(&e);
                        span.set_status(Status::Error {
                            description: Cow::from("msg was sent to dlq"),
                        });
                        return Err(AmqpError::PublishingToDQLError {});
                    }
                };
            }
        }
        _ => {
            match delivery
                .nack(BasicNackOptions {
                    multiple: false,
                    requeue: false,
                })
                .await
            {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!(
                        trace.id = traces::trace_id(&ctx),
                        span.id = traces::span_id(&ctx),
                        "error whiling nack msg"
                    );
                    span.record_error(&e);
                    span.set_status(Status::Error {
                        description: Cow::from("error to nack msg"),
                    });
                    return Err(AmqpError::NackMessageError {});
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockAmqpImpl;
    use async_trait::async_trait;
    use lapin::{acker::Acker, protocol::basic::AMQPProperties, types::ShortString};
    use opentelemetry::Context;

    #[test]
    fn test_dispatch_declare_successfully() {
        let mut dispatcher = Dispatches::new();
        let handler = MockedHandler { mock_error: None };

        let res = dispatcher.declare(
            QueueDefinition::name("queue"),
            "msg_type".to_owned(),
            Arc::new(handler),
        );

        assert!(res.is_ok());
        assert_eq!(dispatcher.handlers.len(), 1);
        assert_eq!(dispatcher.msgs_types.len(), 1);
        assert_eq!(dispatcher.queues.len(), 1);
    }

    #[test]
    fn test_dispatch_declare_error() {
        let mut dispatcher = Dispatches::new();
        let handler = MockedHandler { mock_error: None };

        let res = dispatcher.declare(
            QueueDefinition::name("queue"),
            "".to_owned(),
            Arc::new(handler),
        );

        assert!(res.is_err());
        assert_eq!(dispatcher.handlers.len(), 0);
        assert_eq!(dispatcher.msgs_types.len(), 0);
        assert_eq!(dispatcher.queues.len(), 0);
    }

    #[tokio::test]
    async fn test_consume_msg_correctly() {
        let tracer = global::tracer("test");
        let amqp_mocked = MockAmqpImpl::new();

        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("msg_type")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let handler = Arc::new(MockedHandler { mock_error: None });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            "msg_type",
            &["msg_type".to_owned()],
            &delivery,
            Arc::new(amqp_mocked),
            handler,
        )
        .await;

        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_consume_msg_when_has_no_msg_type() {
        let tracer = global::tracer("test");
        let amqp_mocked = MockAmqpImpl::new();
        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let handler = Arc::new(MockedHandler { mock_error: None });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            "msg_type",
            &["msg_type".to_owned()],
            &delivery,
            Arc::new(amqp_mocked),
            handler,
        )
        .await;

        assert!(res.is_ok());

        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("kind")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let amqp_mocked = MockAmqpImpl::new();
        let handler = Arc::new(MockedHandler { mock_error: None });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            "msg_type",
            &["msg_type".to_owned()],
            &delivery,
            Arc::new(amqp_mocked),
            handler,
        )
        .await;

        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_consume_when_receive_no_expected_msg_type() {
        let tracer = global::tracer("test");
        let amqp_mocked = MockAmqpImpl::new();
        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("msg_type")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let handler = Arc::new(MockedHandler { mock_error: None });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            "msg_type",
            &["msg_type".to_owned()],
            &delivery,
            Arc::new(amqp_mocked),
            handler,
        )
        .await;

        assert!(res.is_ok())
    }
    pub struct MockedHandler {
        pub mock_error: Option<AmqpError>,
    }

    #[tokio::test]
    async fn test_consume_msg_with_handler_error_without_retry() {
        let tracer = global::tracer("test");
        let amqp_mocked = MockAmqpImpl::new();
        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("msg_type")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let handler = Arc::new(MockedHandler {
            mock_error: Some(AmqpError::InternalError {}),
        });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            "msg_type",
            &["msg_type".to_owned()],
            &delivery,
            Arc::new(amqp_mocked),
            handler,
        )
        .await;

        assert!(res.is_ok())
    }

    #[async_trait]
    impl ConsumerHandler for MockedHandler {
        async fn exec(&self, _ctx: &Context, _data: &[u8]) -> Result<(), AmqpError> {
            if self.mock_error.is_none() {
                return Ok(());
            }

            Err(AmqpError::InternalError {})
        }
    }
}
