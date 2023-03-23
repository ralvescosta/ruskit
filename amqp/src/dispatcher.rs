use crate::{consumer::consume, errors::AmqpError, queue::QueueDefinition};
use async_trait::async_trait;
use futures_util::{future::join_all, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
use opentelemetry::{global, Context};
use std::{collections::HashMap, fmt::Debug, sync::Arc, vec};
use tokio::task::JoinError;
use tracing::error;

#[async_trait]
pub trait ConsumerHandler {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError>;
}

#[derive(Clone)]
pub struct DispatcherDefinition {
    pub(crate) queue: String,
    pub(crate) queue_def: QueueDefinition,
    pub(crate) handler: Arc<dyn ConsumerHandler + Send + Sync>,
}

pub trait Dispatcher<'d> {
    fn register<T>(
        self,
        queue: &str,
        msg: T,
        handler: Arc<dyn ConsumerHandler + Send + Sync>,
    ) -> Self
    where
        T: Debug;
    fn queue_definition(self, def: &'d QueueDefinition) -> Self;
    fn queues_definition(self, queues_def: HashMap<&'d str, &'d QueueDefinition>) -> Self;
}

pub struct AmqpDispatcher<'ad> {
    channel: Arc<Channel>,
    pub(crate) queues_def: HashMap<&'ad str, &'ad QueueDefinition>,
    pub(crate) dispatchers_def: HashMap<String, DispatcherDefinition>,
}

impl<'ad> AmqpDispatcher<'ad> {
    pub fn new(channel: Arc<Channel>) -> AmqpDispatcher<'ad> {
        return AmqpDispatcher {
            channel,
            queues_def: HashMap::default(),
            dispatchers_def: HashMap::default(),
        };
    }
}

impl<'ad> Dispatcher<'ad> for AmqpDispatcher<'ad> {
    fn register<T>(
        mut self,
        queue: &str,
        msg: T,
        handler: Arc<dyn ConsumerHandler + Send + Sync>,
    ) -> Self
    where
        T: Debug,
    {
        let queue_def = match self.queues_def.get(queue) {
            Some(d) => d.to_owned().to_owned(),
            _ => {
                panic!("")
            }
        };

        let msg_type = format!("{:?}", msg);

        self.dispatchers_def.insert(
            msg_type.clone(),
            DispatcherDefinition {
                queue: queue.to_owned(),
                queue_def,
                handler,
            },
        );

        self
    }

    fn queue_definition(mut self, def: &'ad QueueDefinition) -> Self {
        self.queues_def.insert(def.name, def);
        self
    }

    fn queues_definition(mut self, queues_def: HashMap<&'ad str, &'ad QueueDefinition>) -> Self {
        self.queues_def = queues_def;
        self
    }
}

impl<'ad> AmqpDispatcher<'ad> {
    pub async fn consume_blocking(&self) -> Vec<Result<(), JoinError>> {
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
