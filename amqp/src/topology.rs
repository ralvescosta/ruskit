use std::{collections::BTreeMap, sync::Arc};

use crate::{client::Amqp, defs};
use async_trait::async_trait;
use errors::amqp::AmqpError;
use lapin::types::{AMQPValue, FieldTable, LongInt, LongString, ShortString};
use opentelemetry::Context;
use tracing::debug;

#[derive(Debug, Clone, Default)]
pub struct QueueBindingDefinition {
    pub exchange: String,
    pub queue: String,
    pub routing_key: String,
}

impl QueueBindingDefinition {
    pub fn new(exchange: &str, queue: &str, routing_key: &str) -> Self {
        QueueBindingDefinition {
            exchange: exchange.to_owned(),
            queue: queue.to_owned(),
            routing_key: routing_key.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueueDefinition {
    pub name: String,
    pub durable: bool,
    pub delete: bool,
    pub exclusive: bool,
    pub ttl: Option<i32>,
    pub bindings: Vec<QueueBindingDefinition>,
    pub with_dlq: bool,
    pub dlq_name: String,
    pub with_retry: bool,
    pub retry_ttl: Option<i32>,
    pub retries: Option<i64>,
}

impl QueueDefinition {
    pub fn name(name: &str) -> QueueDefinition {
        QueueDefinition {
            name: name.to_owned(),
            durable: true,
            delete: false,
            exclusive: false,
            ..Default::default()
        }
    }

    pub fn durable(mut self) -> Self {
        self.durable = true;
        self.delete = false;
        self
    }

    pub fn not_durable(mut self) -> Self {
        self.durable = false;
        self.delete = true;
        self
    }

    pub fn exclusive(mut self) -> Self {
        self.exclusive = true;
        self
    }

    pub fn not_exclusive(mut self) -> Self {
        self.exclusive = false;
        self
    }

    pub fn with_ttl(mut self, milliseconds: i32) -> Self {
        self.ttl = Some(milliseconds);
        self
    }

    pub fn with_dlq(mut self) -> Self {
        self.with_dlq = true;
        self.dlq_name = self.dlq_name();
        self
    }

    pub fn with_retry(mut self, milliseconds: i32, retries: i64) -> Self {
        self.with_retry = true;
        self.retries = Some(retries);
        self.retry_ttl = Some(milliseconds);
        self
    }

    pub fn binding(mut self, exchange: &str, key: &str) -> Self {
        self.bindings.push(QueueBindingDefinition {
            queue: self.name.clone(),
            exchange: exchange.to_owned(),
            routing_key: key.to_owned(),
        });

        self
    }

    pub fn binding_fanout_exchanges(mut self, exchanges: Vec<String>) -> Self {
        for exchange in exchanges {
            self.bindings.push(QueueBindingDefinition {
                queue: self.name.clone(),
                exchange,
                routing_key: "".to_owned(),
            });
        }

        self
    }

    fn dlq_name(&self) -> String {
        format!("{}-dlq", self.name)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ExchangeKind {
    #[default]
    Direct,
    Fanout,
    Topic,
    Headers,
    XMessageDelayed,
}

impl ExchangeKind {
    pub fn try_into(kind: ExchangeKind) -> lapin::ExchangeKind {
        match kind {
            ExchangeKind::Direct => lapin::ExchangeKind::Direct,
            ExchangeKind::Fanout => lapin::ExchangeKind::Fanout,
            ExchangeKind::Headers => lapin::ExchangeKind::Headers,
            ExchangeKind::Topic => lapin::ExchangeKind::Topic,
            ExchangeKind::XMessageDelayed => {
                lapin::ExchangeKind::Custom("x-delayed-message".to_owned())
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExchangeDefinition {
    pub name: String,
    pub durable: bool,
    pub delete: bool,
    pub kind: ExchangeKind,
    pub params: BTreeMap<ShortString, AMQPValue>,
}

impl ExchangeDefinition {
    pub fn name(name: &str) -> Self {
        ExchangeDefinition {
            name: name.to_owned(),
            durable: true,
            delete: false,
            kind: ExchangeKind::default(),
            params: BTreeMap::new(),
        }
    }

    pub fn durable(mut self) -> Self {
        self.durable = true;
        self.delete = false;
        return self;
    }

    pub fn not_durable(mut self) -> Self {
        self.durable = false;
        self.delete = true;
        return self;
    }

    pub fn direct(mut self) -> Self {
        self.kind = ExchangeKind::Direct;
        self
    }

    pub fn fanout(mut self) -> Self {
        self.kind = ExchangeKind::Fanout;
        self
    }

    pub fn header(mut self) -> Self {
        self.kind = ExchangeKind::Headers;
        self
    }

    pub fn topic(mut self) -> Self {
        self.kind = ExchangeKind::Topic;
        self
    }

    pub fn direct_delayed(mut self) -> Self {
        self.kind = ExchangeKind::XMessageDelayed;
        self.params.insert(
            ShortString::from(defs::AMQP_HEADERS_DELAYED_EXCHANGE_TYPE),
            AMQPValue::LongString(LongString::from("direct")),
        );
        self
    }

    pub fn fanout_delayed(mut self) -> Self {
        self.kind = ExchangeKind::XMessageDelayed;
        self.params.insert(
            ShortString::from(defs::AMQP_HEADERS_DELAYED_EXCHANGE_TYPE),
            AMQPValue::LongString(LongString::from("fanout")),
        );
        self
    }
}

#[async_trait]
pub trait ConsumerHandler {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError>;
}

#[derive(Debug, Default, Clone)]
pub struct ConsumerDefinition {
    pub name: String,
    pub queue: String,
    pub msg_type: String,
    pub with_retry: bool,
    pub retries: i64,
    pub with_dlq: bool,
    pub dlq_name: String,
}

impl ConsumerDefinition {
    pub fn name(name: &str) -> ConsumerDefinition {
        ConsumerDefinition {
            name: name.to_owned(),
            retries: 1,
            ..Default::default()
        }
    }

    pub fn queue(mut self, queue: &str) -> Self {
        self.queue = queue.to_owned();
        self
    }

    pub fn msg_type(mut self, msg_type: &str) -> Self {
        self.msg_type = msg_type.to_owned();
        self
    }

    pub fn with_dlq(mut self) -> Self {
        self.with_dlq = true;
        self
    }

    pub fn with_retry(mut self, retries: i64) -> Self {
        self.with_retry = true;
        self.retries = retries;
        self
    }
}

pub struct AmqpTopology {
    amqp_client: Arc<dyn Amqp>,
    pub exchanges: Vec<ExchangeDefinition>,
    pub queues: Vec<QueueDefinition>,
    pub consumers: Vec<ConsumerDefinition>,
}

impl AmqpTopology {
    pub fn new(amqp_client: Arc<dyn Amqp>) -> Self {
        AmqpTopology {
            amqp_client,
            exchanges: vec![],
            queues: vec![],
            consumers: vec![],
        }
    }

    pub fn exchange(mut self, exch: ExchangeDefinition) -> Self {
        self.exchanges.push(exch);
        self
    }

    pub fn queue(mut self, queue: QueueDefinition) -> Self {
        self.queues.push(queue);
        self
    }

    pub async fn install(&self) -> Result<(), AmqpError> {
        for exch in self.exchanges.clone() {
            debug!("creating exchange: {}", exch.name);

            self.amqp_client
                .declare_exchange(
                    &exch.name,
                    exch.kind,
                    exch.delete,
                    exch.durable,
                    false,
                    Some(exch.params),
                )
                .await
                .map_err(|_| AmqpError::DeclareExchangeError(exch.name.to_owned()))?;

            debug!("exchange: {} was created", exch.name);
        }

        for queue in self.queues.clone() {
            debug!("creating and binding queue: {}", queue.name);

            let queue_map = self.install_retry(&queue).await?;
            let mut queue_map = self.install_dlq(&queue, queue_map).await?;

            if queue.ttl.is_some() {
                queue_map.insert(
                    ShortString::from(defs::AMQP_HEADERS_MESSAGE_TTL),
                    AMQPValue::LongInt(LongInt::from(queue.ttl.unwrap())),
                );
            }

            self.amqp_client
                .declare_queue(
                    &queue.name,
                    queue.delete,
                    queue.durable,
                    queue.exclusive,
                    &FieldTable::from(queue_map),
                )
                .await
                .map_err(|_| AmqpError::DeclareQueueError(queue.name.to_owned()))?;

            for bind in queue.clone().bindings {
                self.amqp_client
                    .binding_exchange_queue(&bind.exchange, &bind.queue, &bind.routing_key)
                    .await
                    .map_err(|_| {
                        AmqpError::BindingExchangeToQueueError(
                            bind.exchange.to_owned(),
                            bind.queue.to_owned(),
                        )
                    })?;
            }

            debug!("queue: {} was created and bonded", queue.name);
        }

        Ok(())
    }

    async fn install_dlq(
        &self,
        def: &QueueDefinition,
        queue_map_from_retry: BTreeMap<ShortString, AMQPValue>,
    ) -> Result<BTreeMap<ShortString, AMQPValue>, AmqpError> {
        if !def.with_dlq && !def.with_retry {
            return Ok(BTreeMap::new());
        }

        debug!("creating DLQ...");
        let mut queue_map = queue_map_from_retry;
        let name = self.dlq_name(&def.name);

        if !def.with_retry {
            queue_map.insert(
                ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
                AMQPValue::LongString(LongString::from("")),
            );

            queue_map.insert(
                ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
                AMQPValue::LongString(LongString::from(name.clone())),
            );
        }

        self.amqp_client
            .declare_queue(
                &name,
                def.delete,
                def.durable,
                def.exclusive,
                &FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(name))?;
        debug!("DLQ created");

        Ok(queue_map)
    }

    async fn install_retry(
        &self,
        def: &QueueDefinition,
    ) -> Result<BTreeMap<ShortString, AMQPValue>, AmqpError> {
        if !def.with_retry {
            return Ok(BTreeMap::new());
        }

        debug!("creating retry...");
        let mut retry_map = BTreeMap::new();
        retry_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
            AMQPValue::LongString(LongString::from("")),
        );
        retry_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
            AMQPValue::LongString(LongString::from(def.name.clone())),
        );
        retry_map.insert(
            ShortString::from(defs::AMQP_HEADERS_MESSAGE_TTL),
            AMQPValue::LongInt(LongInt::from(def.retry_ttl.unwrap())),
        );

        let name = self.retry_name(&def.name);
        self.amqp_client
            .declare_queue(
                &name,
                def.delete,
                def.durable,
                def.exclusive,
                &FieldTable::from(retry_map),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(name.clone()))?;

        let mut queue_map = BTreeMap::new();
        queue_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
            AMQPValue::LongString(LongString::from("")),
        );

        queue_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
            AMQPValue::LongString(LongString::from(name)),
        );
        debug!("retry created");

        Ok(queue_map)
    }

    fn retry_name(&self, queue: &str) -> String {
        format!("{}-retry", queue)
    }

    fn dlq_name(&self, queue: &str) -> String {
        format!("{}-dlq", queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockAmqpImpl;

    #[test]
    fn test_queue_binding_definition() {
        let def = QueueBindingDefinition::new("exchange", "queue", "routingKey");

        assert_eq!(def.exchange, "exchange");
        assert_eq!(def.queue, "queue");
        assert_eq!(def.routing_key, "routingKey");
    }

    #[test]
    fn test_queue_definition() {
        let def = QueueDefinition::name("name");
        assert_eq!(def.name, "name");

        let def = def.with_dlq();
        assert!(def.with_dlq);
        assert_eq!(def.dlq_name, "name-dlq");

        let def = def.with_retry(1000, 3);
        assert!(def.with_retry);
        assert_eq!(def.retries, Some(3));
        assert_eq!(def.retry_ttl, Some(1000));

        let binding = QueueBindingDefinition::new("exchange", "queue", "routing_key");
        let def = def.binding("exchange", "key");
        assert_eq!(def.bindings[0].exchange, binding.exchange);
    }

    #[test]
    fn test_exchange_kind() {
        let kind = ExchangeKind::Direct;
        assert_eq!(ExchangeKind::try_into(kind), lapin::ExchangeKind::Direct);

        let kind = ExchangeKind::Fanout;
        assert_eq!(ExchangeKind::try_into(kind), lapin::ExchangeKind::Fanout);

        let kind = ExchangeKind::Headers;
        assert_eq!(ExchangeKind::try_into(kind), lapin::ExchangeKind::Headers);

        let kind = ExchangeKind::Topic;
        assert_eq!(ExchangeKind::try_into(kind), lapin::ExchangeKind::Topic);
    }

    #[test]
    fn test_exchange_definition() {
        let def = ExchangeDefinition::name("exchange");
        assert_eq!(def.name, "exchange");

        let def = def.direct();
        assert_eq!(def.kind, ExchangeKind::Direct);

        let def = def.fanout();
        assert_eq!(def.kind, ExchangeKind::Fanout);

        let def = def.topic();
        assert_eq!(def.kind, ExchangeKind::Topic);

        let def = def.header();
        assert_eq!(def.kind, ExchangeKind::Headers);
    }

    #[test]
    fn test_retry() {
        let def = ConsumerDefinition::name("consumer");
        assert_eq!(def.name, "consumer");

        let def = def.queue("queue");
        assert_eq!(def.queue, "queue");

        let def = def.with_dlq();
        assert!(def.with_dlq);

        let def = def.with_retry(3);
        assert!(def.with_retry);
        assert_eq!(def.retries, 3);
    }

    #[test]
    fn test_amqp_topology() {
        let amqp_mocked = MockAmqpImpl::new();
        let topology = AmqpTopology::new(Arc::new(amqp_mocked));

        let exch_def = ExchangeDefinition::name("exchange");
        let topology = topology.exchange(exch_def.clone());
        assert_eq!(topology.exchanges[0].name, exch_def.name);

        let queue_def = QueueDefinition::name("queue");
        let topology = topology.queue(queue_def.clone());
        assert_eq!(topology.queues[0].name, queue_def.name);

        let queue_def = QueueDefinition::name("queue").with_retry(1000, 3);
        let topology = topology.queue(queue_def.clone());
        assert!(topology.queues.len() > 0);
    }
}
