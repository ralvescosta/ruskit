use crate::{dispatcher::DispatcherDefinition, errors::AmqpError, otel};
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicNackOptions, BasicPublishOptions},
    protocol::basic::AMQPProperties,
    types::FieldTable,
    Channel,
};
use opentelemetry::{
    global::BoxedTracer,
    trace::{Span, Status},
};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tracing::{debug, error, warn};

pub const AMQP_HEADERS_X_DEATH: &str = "x-death";
pub const AMQP_HEADERS_COUNT: &str = "count";

/// Consume a message from RabbitMQ and process it.
///
/// This function takes a `BoxedTracer` for tracing purposes, a `Delivery` message to process,
/// a HashMap of `DispatcherDefinition` objects to find the correct handler for the message type,
/// and an `Arc<Channel>` for communicating with RabbitMQ.
///
/// It extracts the message type and count from the message header properties, creates a new
/// OpenTelemetry span with the `BoxedTracer`, and logs a debug message with the message type.
///
/// If the message type is not supported, it removes the message from the queue, logs an error
/// with the span, sets the span status to error, and returns an `AmqpError::InternalError`.
///
/// If the message type is supported, it executes the corresponding handler and logs a debug
/// message if successful. If the handler fails and there is no retry configured, it acks the
/// message and removes it from the queue. If the handler fails and there is a retry configured,
/// it nacks the message and sends it to the retry queue if the retry count has not reached the
/// maximum. If the retry count has reached the maximum, it sends the message to the dead-letter
/// queue.
///
/// Returns `Ok(())` if the message was processed successfully or an `AmqpError` if there was
/// an error during processing.
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

/// Extracts the message type and retry count from AMQP message properties.
///
/// # Arguments
///
/// * `props` - A reference to the `AMQPProperties` object containing message properties.
///
/// # Returns
///
/// A tuple containing the message type (as a `String`) and retry count (as an `i64`).
///
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
