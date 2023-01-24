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
    global::{BoxedSpan, BoxedTracer},
    trace::{FutureExt, Span, Status},
    Context,
};
use std::{borrow::Cow, sync::Arc};
use tracing::{debug, error, warn};

pub(crate) async fn consume<'c>(
    tracer: &'c BoxedTracer,
    queue: &'c QueueDefinition,
    msgs_allowed: &'c [String],
    handlers: &'c [Arc<dyn ConsumerHandler + Send + Sync>],
    delivery: &'c Delivery,
    amqp: Arc<dyn Amqp + Send + Sync>,
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

    //check if the message received are not expect for any consumer
    check_received_msg_type(&ctx, &mut span, &metadata.msg_type, msgs_allowed, delivery).await?;

    let handler = get_msg_handler(
        &ctx,
        &mut span,
        &metadata.msg_type,
        msgs_allowed,
        handlers,
        delivery,
    )
    .await?;

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

    //ack msg and remove from queue if handler failure and there are no fallback configured or send to dlq
    if (!queue.with_retry && !queue.with_dlq) || (!queue.with_retry && queue.with_dlq) {
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

async fn check_received_msg_type(
    ctx: &Context,
    span: &mut BoxedSpan,
    msg_type: &String,
    msgs_allowed: &[String],
    delivery: &Delivery,
) -> Result<(), AmqpError> {
    if !msg_type.is_empty() || msgs_allowed.contains(msg_type) {
        return Ok(());
    }

    let msg = "removing message from queue - reason: unsupported msg type";

    span.record_error(&AmqpError::ConsumerError(msg.to_string()));
    span.set_status(Status::Error {
        description: Cow::from(msg),
    });

    debug!(
        trace.id = traces::trace_id(&ctx),
        span.id = traces::span_id(&ctx),
        "{}",
        msg
    );

    match delivery.ack(BasicAckOptions { multiple: false }).await {
        Err(e) => {
            error!("error whiling ack msg");

            span.record_error(&e);
            span.set_status(Status::Error {
                description: Cow::from("error to ack msg"),
            });
        }
        _ => {}
    };

    Err(AmqpError::InternalError {})
}

async fn get_msg_handler(
    ctx: &Context,
    span: &mut BoxedSpan,
    msg_type: &String,
    msgs_allowed: &[String],
    handlers: &[Arc<dyn ConsumerHandler + Send + Sync>],
    delivery: &Delivery,
) -> Result<Arc<dyn ConsumerHandler + Send + Sync>, AmqpError> {
    let mut idx: i8 = -1;
    for i in 0..msgs_allowed.len() {
        if msgs_allowed[i].eq(msg_type) {
            idx = i as i8;
            break;
        }
    }

    if idx < 0 {
        match delivery
            .nack(BasicNackOptions {
                multiple: false,
                requeue: false,
            })
            .await
        {
            Ok(_) => {
                return Err(AmqpError::ConsumerError(format!(
                    "handler for {} was not founded",
                    msg_type
                )))
            }
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

    Ok(handlers[idx as usize].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockAmqpImpl;
    use async_trait::async_trait;
    use lapin::{acker::Acker, protocol::basic::AMQPProperties, types::ShortString};
    use opentelemetry::{global, Context};

    #[tokio::test]
    async fn should_consume_msg_correctly() {
        let tracer = global::tracer("test");
        let amqp_mocked = MockAmqpImpl::new();

        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("msg_type3")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let handler = Arc::new(MockedHandler { mock_error: None });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            &[
                "msg_type1".to_owned(),
                "msg_type2".to_owned(),
                "msg_type3".to_owned(),
                "msg_type4".to_owned(),
            ],
            &[
                handler.clone(),
                handler.clone(),
                handler.clone(),
                handler.clone(),
            ],
            &delivery,
            Arc::new(amqp_mocked),
        )
        .await;

        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn should_remove_msg_from_queue_when_msg_has_no_msg_type() {
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
            &["msg_type".to_owned()],
            &[handler],
            &delivery,
            Arc::new(amqp_mocked),
        )
        .await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_remove_msg_from_queue_when_msg_type_is_not_allowed() {
        let tracer = global::tracer("test");
        let amqp_mocked = MockAmqpImpl::new();
        let delivery = Delivery {
            acker: Acker::default(),
            data: vec![],
            delivery_tag: 0,
            exchange: ShortString::from(""),
            properties: AMQPProperties::default().with_kind(ShortString::from("something")),
            redelivered: false,
            routing_key: ShortString::from(""),
        };
        let handler = Arc::new(MockedHandler { mock_error: None });

        let res = consume(
            &tracer,
            &QueueDefinition::name("queue"),
            &["msg_type".to_owned()],
            &[handler],
            &delivery,
            Arc::new(amqp_mocked),
        )
        .await;

        assert!(res.is_err())
    }

    #[tokio::test]
    async fn should_consume_msg_with_error_and_without_retry() {
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
            &["msg_type".to_owned()],
            &[handler],
            &delivery,
            Arc::new(amqp_mocked),
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
