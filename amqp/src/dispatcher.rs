use crate::{consumer::consume, errors::AmqpError, queue::QueueDefinition};
use async_trait::async_trait;
use futures_util::{future::join_all, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
#[cfg(test)]
use mockall::*;
#[cfg(feature = "mocks")]
use mockall::*;
use opentelemetry::{global, Context};
use std::{collections::HashMap, fmt::Display, sync::Arc, vec};
use tokio::task::JoinError;
use tracing::error;

/// Trait implemented by a message handler.
#[cfg_attr(test, automock)]
#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    /// Executes the handler logic for a message.
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError>;
}

/// A definition of a message type and its corresponding queue and handler.
#[derive(Clone)]
pub struct DispatcherDefinition {
    /// The name of the queue that messages of this type are consumed from.
    pub(crate) queue: String,
    /// The definition of the queue that messages of this type are consumed from.
    pub(crate) queue_def: QueueDefinition,
    /// The handler for messages of this
    pub(crate) handler: Arc<dyn ConsumerHandler>,
}

/// Trait to define a dispatcher for the AMQP messages.
///
/// A `Dispatcher` is an abstraction over the RabbitMQ message broker, which allows
/// messages to be consumed from a queue and dispatched to registered handlers. Each handler is
/// responsible for processing messages of a specific type, which is determined by the message
/// type string representation. The dispatcher consumes messages from the registered queues and
/// dispatches them to the corresponding handlers based on the message type string.
///
/// # Example:
///
///
/// ```rust,no_run
/// use amqp::{
///     dispatcher::{AmqpDispatcher, Dispatcher, ConsumerHandler, DispatcherDefinition},
///     queue::QueueDefinition,
///     channel::new_amqp_channel
/// };
/// use configs::Empty;
/// use configs_builder::ConfigsBuilder
/// use async_trait::async_trait;
/// use std::sync::Arc;
/// use tokio::task::JoinError;
/// use tracing::{error, info};
///
/// struct MyHandler {}
///
/// #[async_trait]
/// impl ConsumerHandler for MyHandler {
///     async fn exec(&self, _ctx: &Context, data: &[u8]) -> Result<(), AmqpError> {
///         // Handle message data here.
///         Ok(())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), AmqpError> {
///     // Read configs from .env file
///     let configs = ConfigsBuilder::new().build::<Empty>().await?
///
///     // Create a new channel.
///     let (_conn, channel) = new_amqp_channel(&configs).await?;
///
///     // Define a queue.
///     let queue_def = QueueDefinition::new("queue").durable();
///
///     // Create a new dispatcher.
///     let dispatcher = AmqpDispatcher::new(channel);
///
///     // Register a handler for the "my_message" message type.
///     let dispatcher = dispatcher.register(&queue_def, &"my_message", Arc::new(MyHandler {})).await;
///
///     // Consume messages.
///     let results = dispatcher.consume_blocking().await;
///
///     // Handle any join errors.
///     for result in results {
///         if let Err(err) = result {
///             error!(error = err.to_string(), "error joining task");
///         }
///     }
///
///     info!("shutdown complete");
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait Dispatcher: Send + Sync {
    /// Method to register a message handler for a queue.
    fn register<'dp, T>(
        self,
        def: &'dp QueueDefinition,
        msg: &'dp T,
        handler: Arc<dyn ConsumerHandler>,
    ) -> Self
    where
        T: Display + 'static;

    /// Method to start consuming messages from the queue.
    async fn consume_blocking(&self) -> Vec<Result<(), JoinError>>;
}

/// Struct to define an AMQP dispatcher.
///
/// # Example:
///
///
/// ```rust,no_run
/// use amqp::{
///     dispatcher::AmqpDispatcher,
///     channel::new_amqp_channel
/// };
/// use configs::Empty;
/// use configs_builder::ConfigsBuilder
///
/// #[tokio::main]
/// async fn main() -> Result<(), AmqpError> {
///     // Read configs from .env file
///     let configs = ConfigsBuilder::new().build::<Empty>().await?
///
///     // Create a new channel.
///     let (_conn, channel) = new_amqp_channel(&configs).await?;
///
///     // Create a new dispatcher.
///     let dispatcher = AmqpDispatcher::new(channel);
///
///     Ok(())
/// }
/// ```
pub struct AmqpDispatcher {
    /// Channel to interact with AMQP.
    channel: Arc<Channel>,
    /// Definitions of all dispatchers for queues.
    pub(crate) dispatchers_def: HashMap<String, DispatcherDefinition>,
}

impl AmqpDispatcher {
    /// Method to create a new instance of `AmqpDispatcher`.
    pub fn new(channel: Arc<Channel>) -> AmqpDispatcher {
        return AmqpDispatcher {
            channel,
            dispatchers_def: HashMap::default(),
        };
    }
}

#[async_trait]
impl<'ad> Dispatcher for AmqpDispatcher {
    /// Method to register a message handler for a queue.
    fn register<'dp, T>(
        mut self,
        def: &'dp QueueDefinition,
        msg: &'dp T,
        handler: Arc<dyn ConsumerHandler>,
    ) -> Self
    where
        T: Display,
    {
        self.dispatchers_def.insert(
            format!("{}", msg),
            DispatcherDefinition {
                queue: def.name.to_owned(),
                queue_def: def.to_owned(),
                handler,
            },
        );

        self
    }

    /// Method to start consuming messages from the queue.
    async fn consume_blocking(&self) -> Vec<Result<(), JoinError>> {
        let mut spawns = vec![];

        for (msg_type, def) in self.dispatchers_def.clone() {
            let mut consumer = self
                .channel
                .basic_consume(
                    &def.queue,
                    &msg_type,
                    BasicConsumeOptions {
                        no_local: false,
                        no_ack: false,
                        exclusive: false,
                        nowait: false,
                    },
                    FieldTable::default(),
                )
                .await
                .expect("");

            let defs = self.dispatchers_def.clone();
            let channel = self.channel.clone();

            spawns.push(tokio::spawn({
                async move {
                    while let Some(result) = consumer.next().await {
                        match result {
                            Ok(delivery) => {
                                match consume(
                                    &global::tracer("amqp consumer"),
                                    &delivery,
                                    &defs,
                                    channel.clone(),
                                )
                                .await
                                {
                                    Err(err) => {
                                        error!(error = err.to_string(), "error consume msg")
                                    }
                                    _ => {}
                                }
                            }

                            Err(err) => error!(error = err.to_string(), "errors consume msg"),
                        }
                    }
                }
            }));
        }

        join_all(spawns).await
    }
}
