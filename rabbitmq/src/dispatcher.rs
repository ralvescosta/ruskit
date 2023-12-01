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

#[cfg_attr(test, automock)]
#[cfg_attr(feature = "mocks", automock)]
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
pub trait Dispatcher: Send + Sync {
    fn register<'dp, T>(
        self,
        def: &'dp QueueDefinition,
        msg: &'dp T,
        handler: Arc<dyn ConsumerHandler>,
    ) -> Self
    where
        T: Display + 'static;

    async fn consume_blocking_single(&self) -> Result<(), JoinError>;

    async fn consume_blocking_multi(&self) -> Vec<Result<(), JoinError>>;
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
impl<'ad> Dispatcher for AmqpDispatcher {
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

    async fn consume_blocking_multi(&self) -> Vec<Result<(), JoinError>> {
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

    async fn consume_blocking_single(&self) -> Result<(), JoinError> {
        let key = self.dispatchers_def.keys().next().unwrap();
        let def = self.dispatchers_def.get(key).unwrap();

        let mut consumer = self
            .channel
            .basic_consume(
                &def.queue,
                key,
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

        tokio::spawn({
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
        })
        .await
    }
}
