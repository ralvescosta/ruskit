use crate::{
    client::Amqp,
    topology::{ConsumerHandler, QueueDefinition},
    types::{new_span, Metadata},
};
use errors::amqp::AmqpError;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicNackOptions, BasicPublishOptions},
};
use opentelemetry::{
    global::BoxedTracer,
    trace::{FutureExt, Span, Status},
};
use std::{borrow::Cow, sync::Arc};
use tracing::{debug, error, warn};

pub(crate) async fn consume<'c>(
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

    //check if the received msg contains type and if the type match with the expectation
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
            Err(e) => {
                error!("error whiling nack msg");
                span.record_error(&e);
            }
            _ => {}
        }
        return Ok(());
    };

    //check if the message received is expected for other consumers
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
            Err(e) => {
                error!("error whiling nack msg");
                span.record_error(&e);
            }
            _ => {}
        };
        return Ok(());
    }

    //ack msg and remove from queue if the handler execute correctly
    if let Ok(_) = handler
        .exec(&ctx, delivery.data.as_slice())
        .with_context(ctx.clone())
        .await
    {
        match delivery.ack(BasicAckOptions { multiple: false }).await {
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
            _ => {
                span.set_status(Status::Ok);
                return Ok(());
            }
        }
    };

    //ack msg and remove from queue if handler failure and there are no fallback configured
    if !queue.with_retry && !queue.with_dlq {
        match delivery.ack(BasicAckOptions { multiple: false }).await {
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

    //send msg to dlq if handler failure and there is no retry configured
    if !queue.with_retry && queue.with_dlq {
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
            _ => {
                match delivery.ack(BasicAckOptions { multiple: false }).await {
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
                    _ => return Ok(()),
                };
            }
        };
    }

    //send msg to retry when handler failure and the retry count Dont active the max of the retries configured
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
    }

    //send msg to dlq when count active the max retries
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
        _ => {
            match delivery.ack(BasicAckOptions { multiple: false }).await {
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
                _ => return Ok(()),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockAmqpImpl;
    use async_trait::async_trait;
    use lapin::{acker::Acker, protocol::basic::AMQPProperties, types::ShortString};
    use opentelemetry::{global, Context};

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

    pub struct MockedHandler {
        pub mock_error: Option<AmqpError>,
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
