use crate::otel::RabbitMQTracePropagator;
use async_trait::async_trait;
use lapin::{
    options::BasicPublishOptions,
    types::{
        AMQPValue, FieldTable, LongInt, LongLongInt, LongString, LongUInt, ShortInt, ShortString,
    },
    BasicProperties, Channel,
};
use messaging::{
    errors::MessagingError,
    publisher::{HeaderValues, PublishMessage, Publisher},
};
use opentelemetry::{global, Context};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
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
    async fn publish(&self, ctx: &Context, infos: &PublishMessage) -> Result<(), MessagingError> {
        let mut btree = BTreeMap::<ShortString, AMQPValue>::default();

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(ctx, &mut RabbitMQTracePropagator::new(&mut btree))
        });

        if infos.headers.is_some() {
            self.btree_map(&infos.headers.clone().unwrap(), &mut btree);
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
                &infos.data,
                BasicProperties::default()
                    .with_content_type(ShortString::from(JSON_CONTENT_TYPE))
                    .with_kind(ShortString::from(infos.msg_type.clone()))
                    .with_message_id(ShortString::from(Uuid::new_v4().to_string()))
                    .with_headers(FieldTable::from(btree)),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error publishing message");
                Err(MessagingError::PublisherError)
            }
            _ => Ok(()),
        }
    }
}

impl RabbitMQPublisher {
    fn btree_map(
        &self,
        hash_map: &HashMap<String, HeaderValues>,
        btree: &mut BTreeMap<ShortString, AMQPValue>,
    ) {
        for (key, value) in hash_map.clone() {
            let amqp_value = match value {
                HeaderValues::ShortString(v) => AMQPValue::ShortString(ShortString::from(v)),
                HeaderValues::LongString(v) => AMQPValue::LongString(LongString::from(v)),
                HeaderValues::Int(v) => AMQPValue::ShortInt(ShortInt::from(v)),
                HeaderValues::LongInt(v) => AMQPValue::LongInt(LongInt::from(v)),
                HeaderValues::LongLongInt(v) => AMQPValue::LongLongInt(LongLongInt::from(v)),
                HeaderValues::Uint(v) => AMQPValue::LongUInt(LongUInt::from(v)),
                HeaderValues::LongUint(v) => AMQPValue::LongUInt(LongUInt::from(v)),
                HeaderValues::LongLongUint(v) => AMQPValue::LongUInt(LongUInt::from(v as u32)),
            };

            btree.insert(ShortString::from(key), amqp_value);
        }
    }
}
