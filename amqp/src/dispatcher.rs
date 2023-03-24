use crate::{consumer::consume, errors::AmqpError, queue::QueueDefinition};
use async_trait::async_trait;
use futures_util::{future::join_all, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
use opentelemetry::{global, Context};
use std::{collections::HashMap, fmt::Display, sync::Arc, vec};
use tokio::task::JoinError;
use tracing::error;

#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError>;
}

#[derive(Clone)]
pub struct DispatcherDefinition {
    pub(crate) queue: String,
    pub(crate) queue_def: QueueDefinition,
    pub(crate) handler: Arc<dyn ConsumerHandler>,
}

#[async_trait]
pub trait Dispatcher<'d>: Send + Sync {
    fn register<T>(
        self,
        def: &'d QueueDefinition,
        msg: &'d T,
        handler: Arc<dyn ConsumerHandler>,
    ) -> Self
    where
        T: Display + 'static;

    async fn consume_blocking(&self) -> Vec<Result<(), JoinError>>;
}

pub struct AmqpDispatcher {
    channel: Arc<Channel>,
    pub(crate) dispatchers_def: HashMap<String, DispatcherDefinition>,
}

impl AmqpDispatcher {
    pub fn new(channel: Arc<Channel>) -> AmqpDispatcher {
        return AmqpDispatcher {
            channel,
            dispatchers_def: HashMap::default(),
        };
    }
}

#[async_trait]
impl<'ad> Dispatcher<'ad> for AmqpDispatcher {
    fn register<T>(
        mut self,
        def: &'ad QueueDefinition,
        msg: &'ad T,
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

    async fn consume_blocking(&self) -> Vec<Result<(), JoinError>> {
        let mut spawns = vec![];

        for (msg_type, def) in self.dispatchers_def.clone() {
            let mut consumer = self
                .channel
                .basic_consume(
                    &def.queue,
                    &msg_type,
                    BasicConsumeOptions {
                        no_local: true,
                        no_ack: true,
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
