use crate::{dispatcher::DispatcherDefinition, errors::AmqpError, otel};
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicNackOptions, BasicPublishOptions},
    protocol::basic::AMQPProperties,
    types::FieldTable,
    Channel,
};
use opentelemetry::{
    global::{BoxedSpan, BoxedTracer},
    trace::{FutureExt, Span, Status},
    Context,
};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tracing::{debug, error, warn};

pub const AMQP_HEADERS_X_DEATH: &str = "x-death";
pub const AMQP_HEADERS_COUNT: &str = "count";

pub(crate) async fn consume<'c>(
    tracer: &BoxedTracer,
    delivery: &Delivery,
    defs: &'c HashMap<String, DispatcherDefinition>,
    channel: Arc<Channel>,
) -> Result<(), AmqpError> {
    let (msg_type, count) = extract_header_properties(&delivery.properties);

    let (ctx, mut span) = otel::new_span(&delivery.properties, &tracer, &msg_type);

    debug!(
        trace.id = traces::trace_id(&ctx),
        span.id = traces::span_id(&ctx),
        "received: {} - queue: {}",
        msg_type,
        "queue.name",
    );

    let Some(def)  = defs.get(&msg_type) else {
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
        return Err(AmqpError::InternalError {});
    };

    let result = def.handler.exec(&ctx, delivery.data.as_slice()).await;
    if result.is_ok() {
        debug!("message successfully processed");
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
    }

    //ack msg and remove from queue if handler failure and there are no fallback configured or send to dlq
    if !def.queue_def.retry_name.is_none() {
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
    if count < def.queue_def.retries.unwrap() as i64 {
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

    match channel
        .basic_publish(
            "",
            &def.queue_def.clone().dlq_name.unwrap(),
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

fn extract_header_properties(props: &AMQPProperties) -> (String, i64) {
    let headers = match props.headers() {
        Some(val) => val.to_owned(),
        None => FieldTable::default(),
    };

    let count = match headers.inner().get(AMQP_HEADERS_X_DEATH) {
        Some(value) => match value.as_array() {
            Some(arr) => match arr.as_slice().get(0) {
                Some(value) => match value.as_field_table() {
                    Some(table) => match table.inner().get(AMQP_HEADERS_COUNT) {
                        Some(value) => match value.as_long_long_int() {
                            Some(long) => long,
                            _ => 0,
                        },
                        _ => 0,
                    },
                    _ => 0,
                },
                _ => 0,
            },
            _ => 0,
        },
        _ => 0,
    };

    let msg_type = match props.kind() {
        Some(value) => value.to_string(),
        _ => "".to_owned(),
    };

    (msg_type, count)
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::mocks::MockAmqpImpl;
    // use async_trait::async_trait;
    // use lapin::{acker::Acker, protocol::basic::AMQPProperties, types::ShortString};
    // use opentelemetry::{global, Context};

    // #[tokio::test]
    // async fn should_consume_msg_correctly() {
    //     let tracer = global::tracer("test");
    //     let amqp_mocked = MockAmqpImpl::new();

    //     let delivery = Delivery {
    //         acker: Acker::default(),
    //         data: vec![],
    //         delivery_tag: 0,
    //         exchange: ShortString::from(""),
    //         properties: AMQPProperties::default().with_kind(ShortString::from("msg_type3")),
    //         redelivered: false,
    //         routing_key: ShortString::from(""),
    //     };
    //     let handler = Arc::new(MockedHandler { mock_error: None });

    //     let res = consume(
    //         &tracer,
    //         &QueueDefinition::name("queue"),
    //         &[
    //             "msg_type1".to_owned(),
    //             "msg_type2".to_owned(),
    //             "msg_type3".to_owned(),
    //             "msg_type4".to_owned(),
    //         ],
    //         &[
    //             handler.clone(),
    //             handler.clone(),
    //             handler.clone(),
    //             handler.clone(),
    //         ],
    //         &delivery,
    //         Arc::new(amqp_mocked),
    //     )
    //     .await;

    //     assert!(res.is_ok())
    // }

    // #[tokio::test]
    // async fn should_remove_msg_from_queue_when_msg_has_no_msg_type() {
    //     let tracer = global::tracer("test");
    //     let amqp_mocked = MockAmqpImpl::new();
    //     let delivery = Delivery {
    //         acker: Acker::default(),
    //         data: vec![],
    //         delivery_tag: 0,
    //         exchange: ShortString::from(""),
    //         properties: AMQPProperties::default().with_kind(ShortString::from("")),
    //         redelivered: false,
    //         routing_key: ShortString::from(""),
    //     };
    //     let handler = Arc::new(MockedHandler { mock_error: None });

    //     let res = consume(
    //         &tracer,
    //         &QueueDefinition::name("queue"),
    //         &["msg_type".to_owned()],
    //         &[handler],
    //         &delivery,
    //         Arc::new(amqp_mocked),
    //     )
    //     .await;

    //     assert!(res.is_err());
    // }

    // #[tokio::test]
    // async fn should_remove_msg_from_queue_when_msg_type_is_not_allowed() {
    //     let tracer = global::tracer("test");
    //     let amqp_mocked = MockAmqpImpl::new();
    //     let delivery = Delivery {
    //         acker: Acker::default(),
    //         data: vec![],
    //         delivery_tag: 0,
    //         exchange: ShortString::from(""),
    //         properties: AMQPProperties::default().with_kind(ShortString::from("something")),
    //         redelivered: false,
    //         routing_key: ShortString::from(""),
    //     };
    //     let handler = Arc::new(MockedHandler { mock_error: None });

    //     let res = consume(
    //         &tracer,
    //         &QueueDefinition::name("queue"),
    //         &["msg_type".to_owned()],
    //         &[handler],
    //         &delivery,
    //         Arc::new(amqp_mocked),
    //     )
    //     .await;

    //     assert!(res.is_err())
    // }

    // #[tokio::test]
    // async fn should_consume_msg_with_error_and_without_retry() {
    //     let tracer = global::tracer("test");
    //     let amqp_mocked = MockAmqpImpl::new();
    //     let delivery = Delivery {
    //         acker: Acker::default(),
    //         data: vec![],
    //         delivery_tag: 0,
    //         exchange: ShortString::from(""),
    //         properties: AMQPProperties::default().with_kind(ShortString::from("msg_type")),
    //         redelivered: false,
    //         routing_key: ShortString::from(""),
    //     };
    //     let handler = Arc::new(MockedHandler {
    //         mock_error: Some(AmqpError::InternalError {}),
    //     });

    //     let res = consume(
    //         &tracer,
    //         &QueueDefinition::name("queue"),
    //         &["msg_type".to_owned()],
    //         &[handler],
    //         &delivery,
    //         Arc::new(amqp_mocked),
    //     )
    //     .await;

    //     assert!(res.is_ok())
    // }

    // pub struct MockedHandler {
    //     pub mock_error: Option<AmqpError>,
    // }

    // #[async_trait]
    // impl ConsumerHandler for MockedHandler {
    //     async fn exec(&self, _ctx: &Context, _data: &[u8]) -> Result<(), AmqpError> {
    //         if self.mock_error.is_none() {
    //             return Ok(());
    //         }

    //         Err(AmqpError::InternalError {})
    //     }
    // }
}
