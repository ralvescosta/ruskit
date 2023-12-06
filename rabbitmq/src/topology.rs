use crate::{
    errors::AmqpError,
    exchange::{ExchangeBinding, ExchangeDefinition},
    queue::{QueueBinding, QueueDefinition},
};
use async_trait::async_trait;
use lapin::{
    options::{QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable, LongInt, LongString, ShortString},
    Channel,
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tracing::{debug, error};

pub const AMQP_HEADERS_DEAD_LETTER_EXCHANGE: &str = "x-dead-letter-exchange";
pub const AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY: &str = "x-dead-letter-routing-key";
pub const AMQP_HEADERS_MESSAGE_TTL: &str = "x-message-ttl";

#[async_trait]
pub trait Topology<'tp> {
    fn exchange(self, def: &'tp ExchangeDefinition) -> Self;
    fn queue(self, def: &'tp QueueDefinition) -> Self;
    fn exchange_binding(self, binding: &'tp ExchangeBinding) -> Self;
    fn queue_binding(self, binding: &'tp QueueBinding) -> Self;
    async fn install(&self) -> Result<(), AmqpError>;
}

pub struct AmqpTopology<'tp> {
    channel: Arc<Channel>,
    pub(crate) queues: HashMap<&'tp str, &'tp QueueDefinition>,
    pub(crate) queues_binding: Vec<&'tp QueueBinding<'tp>>,
    pub(crate) exchanges: Vec<&'tp ExchangeDefinition<'tp>>,
    pub(crate) exchanges_binding: Vec<&'tp ExchangeBinding>,
}

impl<'tp> AmqpTopology<'tp> {
    pub fn new(channel: Arc<Channel>) -> AmqpTopology<'tp> {
        AmqpTopology {
            channel,
            queues: HashMap::default(),
            queues_binding: vec![],
            exchanges: vec![],
            exchanges_binding: vec![],
        }
    }
}

#[async_trait]
impl<'tp> Topology<'tp> for AmqpTopology<'tp> {
    fn exchange(mut self, def: &'tp ExchangeDefinition) -> Self {
        self.exchanges.push(def);
        self
    }

    fn queue(mut self, def: &'tp QueueDefinition) -> Self {
        self.queues.insert(&def.name, def);
        self
    }

    fn exchange_binding(mut self, binding: &'tp ExchangeBinding) -> Self {
        self.exchanges_binding.push(binding);
        self
    }

    fn queue_binding(mut self, binding: &'tp QueueBinding) -> Self {
        self.queues_binding.push(binding);
        self
    }

    async fn install(&self) -> Result<(), AmqpError> {
        self.install_exchange().await?;
        self.install_queue().await?;
        self.binding_exchanges().await?;
        self.binding_queues().await
    }
}

impl<'tp> AmqpTopology<'tp> {
    async fn install_exchange(&self) -> Result<(), AmqpError> {
        for exch in self.exchanges.clone() {
            debug!("creating exchange: {}", exch.name);

            match self
                .channel
                .exchange_declare(
                    exch.name,
                    exch.kind.clone().try_into().unwrap(),
                    lapin::options::ExchangeDeclareOptions {
                        passive: exch.passive,
                        durable: exch.durable,
                        auto_delete: exch.delete,
                        internal: exch.internal,
                        nowait: exch.no_wait,
                    },
                    FieldTable::from(exch.params.clone()),
                )
                .await
            {
                Err(err) => {
                    error!(
                        error = err.to_string(),
                        name = exch.name,
                        "error to declare the exchange"
                    );
                    Err(AmqpError::DeclareExchangeError(err.to_string()))
                }
                _ => Ok(()),
            }?;

            debug!("exchange: {} was created", exch.name);
        }

        Ok(())
    }

    async fn install_queue(&self) -> Result<(), AmqpError> {
        for (name, def) in self.queues.clone() {
            debug!("creating queue: {}", name);

            let mut queue_args = BTreeMap::new();

            if def.retry_name.is_some() {
                self.declare_retry(def, &mut queue_args).await?;
            }

            if def.dlq_name.is_some() {
                self.declare_dql(def, &mut queue_args).await?;
            }

            if def.ttl.is_some() {
                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_MESSAGE_TTL),
                    AMQPValue::LongInt(LongInt::from(def.ttl.unwrap())),
                );
            }

            match self
                .channel
                .queue_declare(
                    name,
                    QueueDeclareOptions {
                        passive: def.passive,
                        durable: def.durable,
                        exclusive: def.exclusive,
                        auto_delete: def.delete,
                        nowait: def.no_wait,
                    },
                    FieldTable::from(queue_args),
                )
                .await
            {
                Err(err) => {
                    error!(error = err.to_string(), "");
                    Err(AmqpError::DeclareQueueError(name.to_owned()))
                }
                _ => {
                    debug!("queue: {} was created", name);
                    Ok(())
                }
            }?;
        }

        Ok(())
    }

    async fn declare_retry(
        &self,
        def: &QueueDefinition,
        queue_args: &mut BTreeMap<ShortString, AMQPValue>,
    ) -> Result<(), AmqpError> {
        let mut args = BTreeMap::new();

        args.insert(
            ShortString::from(AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
            AMQPValue::LongString(LongString::from("")),
        );
        args.insert(
            ShortString::from(AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
            AMQPValue::LongString(LongString::from(def.name.clone())),
        );
        args.insert(
            ShortString::from(AMQP_HEADERS_MESSAGE_TTL),
            AMQPValue::LongInt(LongInt::from(def.retry_ttl.unwrap())),
        );

        let retry_name = def.retry_name.clone().unwrap();

        match self
            .channel
            .queue_declare(
                &retry_name,
                QueueDeclareOptions {
                    passive: def.passive,
                    durable: def.durable,
                    exclusive: def.exclusive,
                    auto_delete: def.delete,
                    nowait: def.no_wait,
                },
                FieldTable::from(args),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "failure to declare retry queue");
                Err(AmqpError::DeclareQueueError(retry_name))
            }
            _ => {
                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
                    AMQPValue::LongString(LongString::from("")),
                );

                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
                    AMQPValue::LongString(LongString::from(retry_name)),
                );
                Ok(())
            }
        }
    }

    async fn declare_dql(
        &self,
        def: &QueueDefinition,
        queue_args: &mut BTreeMap<ShortString, AMQPValue>,
    ) -> Result<(), AmqpError> {
        let dlq_name = def.dlq_name.clone().unwrap();

        match self
            .channel
            .queue_declare(
                &dlq_name,
                QueueDeclareOptions {
                    passive: def.passive,
                    durable: def.durable,
                    exclusive: def.exclusive,
                    auto_delete: def.delete,
                    nowait: def.no_wait,
                },
                FieldTable::default(),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "failure to declare retry queue");
                Err(AmqpError::DeclareQueueError(dlq_name))
            }
            _ => {
                if def.retry_name.is_none() {
                    queue_args.insert(
                        ShortString::from(AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
                        AMQPValue::LongString(LongString::from("")),
                    );

                    queue_args.insert(
                        ShortString::from(AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
                        AMQPValue::LongString(LongString::from(def.name.clone())),
                    );
                }
                Ok(())
            }
        }
    }

    async fn binding_exchanges(&self) -> Result<(), AmqpError> {
        Ok(())
    }

    async fn binding_queues(&self) -> Result<(), AmqpError> {
        for binding in self.queues_binding.clone() {
            debug!(
                "binding queue: {} to the exchange: {} with the key: {}",
                binding.queue_name, binding.exchange_name, binding.routing_key
            );

            match self
                .channel
                .queue_bind(
                    binding.queue_name,
                    binding.exchange_name,
                    binding.routing_key,
                    QueueBindOptions { nowait: false },
                    FieldTable::default(),
                )
                .await
            {
                Err(err) => {
                    error!(error = err.to_string(), "error to bind queue to exchange");

                    Err(AmqpError::BindingExchangeToQueueError(
                        binding.exchange_name.to_owned(),
                        binding.queue_name.to_owned(),
                    ))
                }
                _ => Ok(()),
            }?;
        }

        debug!("queue was bounded");

        Ok(())
    }
}
