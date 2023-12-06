use crate::otel::AmqpTracePropagator;
use async_trait::async_trait;
use lapin::{
    options::BasicPublishOptions,
    types::{AMQPValue, FieldTable, LongInt, LongString, ShortInt, ShortString},
    BasicProperties, Channel,
};
use messaging::{
    errors::MessagingError,
    publisher::{HeaderValues, PublishInfos, Publisher},
};
use opentelemetry::{global, Context};
use std::{collections::BTreeMap, sync::Arc};
use tracing::error;
use uuid::Uuid;

pub const JSON_CONTENT_TYPE: &str = "application/json";

pub struct RabbitMQPublisher {
    channel: Arc<Channel>,
}

impl RabbitMQPublisher {
    pub fn new(channel: Arc<Channel>) -> Arc<RabbitMQPublisher> {
        Arc::new(RabbitMQPublisher { channel })
    }
}

#[async_trait]
impl Publisher for RabbitMQPublisher {
    async fn publish(&self, ctx: &Context, infos: &PublishInfos) -> Result<(), MessagingError> {
        let mut params = BTreeMap::<ShortString, AMQPValue>::default();

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(ctx, &mut AmqpTracePropagator::new(&mut params))
        });

        if infos.headers.is_some() {
            for (key, value) in infos.headers.clone().unwrap() {
                let amqp_value = match value {
                    HeaderValues::ShortString(v) => AMQPValue::ShortString(ShortString::from(v)),
                    HeaderValues::LongString(v) => AMQPValue::LongString(LongString::from(v)),
                    HeaderValues::Int(v) => AMQPValue::ShortInt(ShortInt::from(v)),
                    HeaderValues::LongInt(v) => AMQPValue::LongInt(LongInt::from(v)),
                };

                params.insert(ShortString::from(key), amqp_value);
            }
        }

        match self
            .channel
            .basic_publish(
                &infos.to,
                &infos.key,
                BasicPublishOptions {
                    immediate: false,
                    mandatory: false,
                },
                &infos.payload,
                BasicProperties::default()
                    .with_content_type(ShortString::from(JSON_CONTENT_TYPE))
                    .with_kind(ShortString::from(infos.msg_type.clone()))
                    .with_message_id(ShortString::from(Uuid::new_v4().to_string()))
                    .with_headers(FieldTable::from(params)),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error publishing message");
                Err(MessagingError::PublishingError)
            }
            _ => Ok(()),
        }
    }
}
