use crate::{errors::AmqpError, queue::QueueDefinition};
use async_trait::async_trait;
use futures_util::StreamExt;
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
use opentelemetry::Context;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::task::JoinError;
use tracing::error;

#[async_trait]
pub trait ConsumerHandler {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError>;
}

pub struct DispatcherDefinition<'dd> {
    pub(crate) queue: &'dd str,
    pub(crate) msg_type: &'dd str,
    pub(crate) queue_def: &'dd QueueDefinition<'dd>,
    pub(crate) handler: Arc<dyn ConsumerHandler>,
}

pub trait Dispatcher<'ad> {
    fn register<T>(self, queue: &'ad str, msg: T, handler: Arc<dyn ConsumerHandler>) -> Self
    where
        T: Debug + Default;
    fn queue_definition(self, def: &'ad QueueDefinition) -> Self;
    fn queues_definition(self, queues_def: HashMap<&'ad str, &'ad QueueDefinition<'ad>>) -> Self;
}

pub struct AmqpDispatcher<'ad> {
    channel: Arc<Channel>,
    pub(crate) queues_def: HashMap<&'ad str, &'ad QueueDefinition<'ad>>,
    pub(crate) dispatchers_def: Vec<DispatcherDefinition<'ad>>,
}

impl<'ad> AmqpDispatcher<'ad> {
    pub fn new(channel: Arc<Channel>) -> AmqpDispatcher<'ad> {
        return AmqpDispatcher {
            channel,
            queues_def: HashMap::default(),
            dispatchers_def: vec![],
        };
    }
}

impl<'ad> Dispatcher<'ad> for AmqpDispatcher<'ad> {
    fn register<T>(mut self, queue: &'ad str, msg: T, handler: Arc<dyn ConsumerHandler>) -> Self
    where
        T: Debug + Default,
    {
        let queue_def = match self.queues_def.get(queue) {
            Some(d) => d.to_owned(),
            _ => {
                panic!("")
            }
        };

        let msg_type = format!("{:?}", msg);

        self.dispatchers_def.push(DispatcherDefinition {
            queue,
            msg_type: "",
            queue_def,
            handler,
        });

        self
    }

    fn queue_definition(mut self, def: &'ad QueueDefinition) -> Self {
        self.queues_def.insert(def.name, def);
        self
    }

    fn queues_definition(
        mut self,
        queues_def: HashMap<&'ad str, &'ad QueueDefinition<'ad>>,
    ) -> Self {
        self.queues_def = queues_def;
        self
    }
}

impl<'ad> AmqpDispatcher<'ad> {
    pub async fn consume_blocking(&self) -> Vec<Result<(), JoinError>> {
        let mut spawns = vec![];

        for def in &self.dispatchers_def {
            spawns.push(tokio::spawn({
                let mut consumer = self
                    .channel
                    .basic_consume(
                        def.queue,
                        def.msg_type,
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

                async move {
                    while let Some(result) = consumer.next().await {
                        match result {
                            Ok(delivery) => {}
                            Err(err) => error!(error = err.to_string(), "errors consume msg"),
                        }
                    }
                }
            }));
        }

        vec![]
    }
}
