use crate::{errors::AmqpError, otel::AmqpTracePropagator};
use async_trait::async_trait;
use lapin::{
    options::BasicPublishOptions,
    types::{AMQPValue, FieldTable, ShortString},
    BasicProperties, Channel,
};
#[cfg(test)]
use mockall::*;
#[cfg(feature = "mocks")]
use mockall::*;
use opentelemetry::{global, Context};
use serde::Serialize;
use std::{collections::BTreeMap, fmt::Display, sync::Arc};
use tracing::error;
use uuid::Uuid;

pub const AMQP_JSON_CONTENT_TYPE: &str = "application/json";

pub struct Payload {
    pub payload: Box<[u8]>,
    pub typ: String,
}

impl Payload {
    pub fn new<T>(p: &T) -> Result<Self, AmqpError>
    where
        T: Display + Serialize,
    {
        match serde_json::to_vec::<T>(p) {
            Ok(c) => Ok(Payload {
                payload: c.into_boxed_slice(),
                typ: format!("{}", p),
            }),
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error to serialize the publish message"
                );
                Err(AmqpError::ParsePayloadError)
            }
        }
    }
}

#[cfg_attr(test, automock)]
#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait Publisher: Send + Sync {
    /// Publishes a message to a target.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The current tracing context.
    /// * `target` - The name of the target queue or exchange.
    /// * `payload` - The payload of the message.
    /// * `params` - The additional parameters for the message (e.g., message headers).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ruskit::{
    ///     configs::Empty,
    ///     configs_builder::ConfigsBuilder,
    ///     amqp::{
    ///         channel:new_amqp_channel,
    ///         publisher::{AmqpPublisher, Payload}
    ///     }
    /// };
    /// use opentelemetry::Context;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Read configs from env file.
    ///     let configs = ConfigsBuilder::new().build::<Empty>();
    ///     
    ///     // Create amqp channel
    ///     let (_, channel) = new_amqp_channel().await?;
    ///
    ///     // Create the publisher
    ///     let publisher = AmqpPublisher::new(channel);
    ///
    ///     let target = "my-exchange";
    ///     let payload = Payload::new(&"{ \"message\": \"hello world\" }".to_owned());
    ///
    ///     publisher.simple_publish(&Context::new(), target, &payload, None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    async fn simple_publish<'btm>(
        &self,
        ctx: &Context,
        target: &str,
        payload: &Payload,
        params: Option<&'btm BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>;

    /// Publishes a message to an exchange.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The current tracing context.
    /// * `exchange` - The name of the exchange.
    /// * `key` - The routing key.
    /// * `payload` - The payload of the message.
    /// * `params` - The additional parameters for the message (e.g., message headers).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ruskit::{
    ///     configs::Empty,
    ///     configs_builder::ConfigsBuilder,
    ///     amqp::{
    ///         channel:new_amqp_channel,
    ///         publisher::{AmqpPublisher, Payload}
    ///     }
    /// };
    /// use opentelemetry::Context;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Read configs from env file.
    ///     let configs = ConfigsBuilder::new().build::<Empty>();
    ///     
    ///     // Create amqp channel
    ///     let (_, channel) = new_amqp_channel().await?;
    ///
    ///     // Create the publisher
    ///     let publisher = AmqpPublisher::new(channel);
    ///
    ///     let exchange = "my-exchange";
    ///     let routing_key = "my-routing-key";
    ///     let payload = Payload::new(&"{ \"message\": \"hello world\" }".to_owned());
    ///
    ///     publisher.publish(&Context::new(), exchange, routing_key, &payload, None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    async fn publish<'btm>(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        payload: &Payload,
        params: Option<&'btm BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>;
}

/// An AMQP publisher for publishing messages to RabbitMQ.
///
/// The `AmqpPublisher` struct implements the `Publisher` trait, providing a simple interface
/// for publishing messages to an AMQP exchange through a RabbitMQ channel.
///
/// # Examples
///
/// Create a new `AmqpPublisher` and use it to publish a message to an exchange:
///
/// ```rust,no_run
/// use ruskit::{
///     configs::Empty,
///     configs_builder::ConfigsBuilder,
///     amqp::{
///         channel:new_amqp_channel,
///         publisher::{AmqpPublisher, Payload}
///     }
/// };
/// use opentelemetry::Context;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Read configs from env file.
///     let configs = ConfigsBuilder::new().build::<Empty>();
///     
///     // Create amqp channel
///     let (_, channel) = new_amqp_channel().await?;
///
///     let publisher = AmqpPublisher::new(channel);
///
///     // Create a payload to publish.
///     let payload = Payload::new(&"{ \"message\": \"hello world\" }".to_owned());
///
///     // Publish the message to an exchange.
///     publisher
///         .publish(&Context::new(), "my_exchange", "my_routing_key", &payload, None)
///         .await?;
///
///     Ok(())
/// }
/// ```
///
/// The `AmqpPublisher` struct can also be used to perform a simple publish to a target destination
/// using the `simple_publish` method.
///
pub struct AmqpPublisher {
    channel: Arc<Channel>,
}

impl AmqpPublisher {
    pub fn new(channel: Arc<Channel>) -> Arc<AmqpPublisher> {
        Arc::new(AmqpPublisher { channel })
    }
}

#[async_trait]
impl Publisher for AmqpPublisher {
    async fn simple_publish<'btm>(
        &self,
        ctx: &Context,
        target: &str,
        payload: &Payload,
        params: Option<&'btm BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError> {
        self.basic(ctx, target, "", payload, params).await
    }

    async fn publish<'btm>(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        payload: &Payload,
        params: Option<&'btm BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError> {
        self.basic(ctx, exchange, key, payload, params).await
    }
}

impl AmqpPublisher {
    async fn basic(
        &self,
        ctx: &Context,
        exchange: &str,
        routing_key: &str,
        payload: &Payload,
        params: Option<&BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError> {
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
                &payload.payload,
                BasicProperties::default()
                    .with_content_type(ShortString::from(AMQP_JSON_CONTENT_TYPE))
                    .with_kind(ShortString::from(payload.typ.clone()))
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
