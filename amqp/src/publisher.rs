use crate::{errors::AmqpError, otel::AmqpTracePropagator};
use async_trait::async_trait;
use lapin::{
    options::BasicPublishOptions,
    types::{AMQPValue, FieldTable, ShortString},
    BasicProperties, Channel,
};
use opentelemetry::{global, Context};
use serde::Serialize;
use std::{collections::BTreeMap, fmt::Debug, sync::Arc};
use tracing::error;
use uuid::Uuid;

pub const AMQP_JSON_CONTENT_TYPE: &str = "application/json";

#[async_trait]
pub trait Publisher<'ap> {
    async fn simple_publish<T>(
        &self,
        ctx: &Context,
        target: &str,
        msg: &'ap T,
        params: Option<&'ap BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>
    where
        T: Debug + Serialize + Send + Sync;

    async fn publish<T>(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        msg: &'ap T,
        params: Option<&'ap BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>
    where
        T: Debug + Serialize + Send + Sync;
}

pub struct AmqpPublisher {
    channel: Arc<Channel>,
}

impl AmqpPublisher {
    pub fn new(channel: Arc<Channel>) -> Arc<AmqpPublisher> {
        Arc::new(AmqpPublisher { channel })
    }
}

#[async_trait]
impl<'ap> Publisher<'ap> for AmqpPublisher {
    async fn simple_publish<T>(
        &self,
        ctx: &Context,
        target: &str,
        msg: &'ap T,
        params: Option<&'ap BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>
    where
        T: Debug + Serialize + Send + Sync,
    {
        self.basic(ctx, target, "", msg, params).await
    }

    async fn publish<T>(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        msg: &'ap T,
        params: Option<&'ap BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>
    where
        T: Debug + Serialize + Send + Sync,
    {
        self.basic(ctx, exchange, key, msg, params).await
    }
}

impl AmqpPublisher {
    async fn basic<'ap, T>(
        &self,
        ctx: &Context,
        exchange: &str,
        routing_key: &str,
        msg: &'ap T,
        params: Option<&'ap BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>
    where
        T: Debug + Serialize + Send + Sync,
    {
        let data = match serde_json::to_vec::<T>(msg) {
            Ok(c) => Ok(c.into_boxed_slice()),
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error to serialize the publish message"
                );
                Err(AmqpError::ParsePayloadError)
            }
        }?;

        let mut params = params
            .unwrap_or(&BTreeMap::<ShortString, AMQPValue>::default())
            .to_owned();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(ctx, &mut AmqpTracePropagator::new(&mut params))
        });

        match self
            .channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions {
                    immediate: false,
                    mandatory: false,
                },
                &data,
                BasicProperties::default()
                    .with_content_type(ShortString::from(AMQP_JSON_CONTENT_TYPE))
                    .with_kind(ShortString::from(format!("{:?}", msg)))
                    .with_message_id(ShortString::from(Uuid::new_v4().to_string()))
                    .with_headers(FieldTable::from(params)),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error publishing message");
                Err(AmqpError::PublishingError)
            }
            _ => Ok(()),
        }
    }
}
