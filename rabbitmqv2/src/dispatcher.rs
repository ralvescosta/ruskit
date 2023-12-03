use crate::{consumer::consume, queue::QueueDefinition};
use async_trait::async_trait;
use futures_util::{future::join_all, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
use messaging::{
    dispatcher::{Dispatcher, DispatcherDefinition},
    errors::MessagingError,
    handler::ConsumerHandler,
};
use opentelemetry::global;
use std::{collections::HashMap, sync::Arc};
use tracing::error;

#[derive(Clone)]
pub struct RabbitMQDispatcherDefinition {
    pub(crate) queue_def: QueueDefinition,
    pub(crate) handler: Arc<dyn ConsumerHandler>,
}

pub struct RabbitMQDispatcher {
    channel: Arc<Channel>,
    queues_def: Vec<QueueDefinition>,
    pub(crate) dispatchers_def: HashMap<String, RabbitMQDispatcherDefinition>,
}

impl RabbitMQDispatcher {
    pub fn new(channel: Arc<Channel>, queues_def: Vec<QueueDefinition>) -> Self {
        RabbitMQDispatcher {
            channel,
            queues_def,
            dispatchers_def: HashMap::default(),
        }
    }
}

#[async_trait]
impl Dispatcher for RabbitMQDispatcher {
    fn register(mut self, def: &DispatcherDefinition, handler: Arc<dyn ConsumerHandler>) -> Self {
        let mut queue_def = QueueDefinition::default();
        for queue in &self.queues_def {
            if def.name == queue.name {
                queue_def = queue.clone();
            }
        }

        self.dispatchers_def.insert(
            def.msg_type.clone(),
            RabbitMQDispatcherDefinition { queue_def, handler },
        );

        self
    }

    async fn consume_blocking(&self) -> Result<(), MessagingError> {
        self.consume_blocking_single().await
    }
}

impl RabbitMQDispatcher {
    pub async fn consume_blocking_single(&self) -> Result<(), MessagingError> {
        let key = self.dispatchers_def.keys().next().unwrap();
        let def = self.dispatchers_def.get(key).unwrap();

        let mut consumer = match self
            .channel
            .basic_consume(
                &def.queue_def.name,
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
        {
            Err(err) => {
                error!(error = err.to_string(), "error to create the consumer");
                Err(MessagingError::CreatingConsumerError)
            }
            Ok(c) => Ok(c),
        }?;

        let defs = self.dispatchers_def.clone();
        let channel = self.channel.clone();

        let spawned = tokio::spawn({
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
        .await;

        if spawned.is_err() {
            return Err(MessagingError::ConsumerError("some error occur".to_owned()));
        }

        return Ok(());
    }

    pub async fn consume_blocking_multi(&self) -> Result<(), MessagingError> {
        let mut spawns = vec![];

        for (msg_type, def) in &self.dispatchers_def {
            let mut consumer = match self
                .channel
                .basic_consume(
                    &def.queue_def.name,
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
            {
                Err(err) => {
                    error!(error = err.to_string(), "failure to create the consumer");
                    Err(MessagingError::CreatingConsumerError)
                }
                Ok(c) => Ok(c),
            }?;

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

        let spawned = join_all(spawns).await;
        for res in spawned {
            if res.is_err() {
                error!("tokio process error");
                return Err(MessagingError::InternalError);
            }
        }

        Ok(())
    }
}
