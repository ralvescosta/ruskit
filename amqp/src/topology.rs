use async_trait::async_trait;
use errors::amqp::AmqpError;
use opentelemetry::Context;

#[derive(Debug, Clone, Copy, Default)]
pub struct QueueBindingDefinition<'q> {
    pub exchange: &'q str,
    pub queue: &'q str,
    pub routing_key: &'q str,
}

impl<'q> QueueBindingDefinition<'q> {
    pub fn new(exchange: &'q str, queue: &'q str, routing_key: &'q str) -> Self {
        QueueBindingDefinition {
            exchange,
            queue,
            routing_key,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueueDefinition<'q> {
    pub name: &'q str,
    pub bindings: Vec<QueueBindingDefinition<'q>>,
    pub with_dlq: bool,
    pub dlq_name: &'q str,
    pub with_retry: bool,
    pub retry_ttl: Option<i32>,
    pub retries: Option<i64>,
}

impl<'q> QueueDefinition<'q> {
    pub fn name(name: &'q str) -> QueueDefinition<'q> {
        QueueDefinition {
            name,
            ..Default::default()
        }
    }

    pub fn with_dlq(mut self) -> Self {
        self.with_dlq = true;
        self.dlq_name = Box::leak(Box::new(self.dlq_name()));
        self
    }

    pub fn with_retry(mut self, milliseconds: i32, retries: i64) -> Self {
        self.with_retry = true;
        self.retries = Some(retries);
        self.retry_ttl = Some(milliseconds);
        self
    }

    pub fn binding(mut self, exchange: &'q str, key: &'q str) -> Self {
        self.bindings.push(QueueBindingDefinition {
            exchange,
            queue: self.name,
            routing_key: key,
        });
        self
    }

    pub fn binding_fanout_exchanges(mut self, exchanges: Vec<&'q str>) -> Self {
        for exchange in exchanges {
            self.bindings.push(QueueBindingDefinition {
                exchange,
                queue: self.name,
                routing_key: "",
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
}

impl ExchangeKind {
    pub fn map(kind: ExchangeKind) -> lapin::ExchangeKind {
        match kind {
            ExchangeKind::Direct => lapin::ExchangeKind::Direct,
            ExchangeKind::Fanout => lapin::ExchangeKind::Fanout,
            ExchangeKind::Headers => lapin::ExchangeKind::Headers,
            ExchangeKind::Topic => lapin::ExchangeKind::Topic,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExchangeDefinition<'e> {
    pub name: &'e str,
    pub kind: ExchangeKind,
}

impl<'e> ExchangeDefinition<'e> {
    pub fn name(name: &'e str) -> Self {
        ExchangeDefinition {
            name,
            kind: ExchangeKind::default(),
        }
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
}

#[async_trait]
pub trait ConsumerHandler {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConsumerDefinition<'c> {
    pub name: &'c str,
    pub queue: &'c str,
    pub msg_type: &'c str,
    pub with_retry: bool,
    pub retries: i64,
    pub with_dlq: bool,
    pub dlq_name: &'c str,
}

impl<'c> ConsumerDefinition<'c> {
    pub fn name(name: &str) -> ConsumerDefinition {
        ConsumerDefinition {
            name,
            retries: 1,
            ..Default::default()
        }
    }

    pub fn queue(mut self, queue: &'c str) -> Self {
        self.queue = queue;
        self
    }

    pub fn msg_type(mut self, msg_type: &'c str) -> Self {
        self.msg_type = msg_type;
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

pub struct AmqpTopology<'a> {
    pub exchanges: Vec<ExchangeDefinition<'a>>,
    pub queues: Vec<QueueDefinition<'a>>,
    pub consumers: Vec<ConsumerDefinition<'a>>,
}

impl<'a> AmqpTopology<'a> {
    pub fn new() -> Self {
        AmqpTopology {
            exchanges: vec![],
            queues: vec![],
            consumers: vec![],
        }
    }

    pub fn exchange(mut self, exch: ExchangeDefinition<'a>) -> Self {
        self.exchanges.push(exch);
        self
    }

    pub fn queue(mut self, queue: QueueDefinition<'a>) -> Self {
        self.queues.push(queue);
        self
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    pub fn consumer_def(
        &self,
        queue_name: &'a str,
        msg_type: &'a str,
    ) -> Option<ConsumerDefinition<'a>> {
        for queue in self.queues.clone() {
            if queue.name == queue_name {
                let retries = match queue.retries {
                    Some(r) => r,
                    _ => 0,
                };
                return Some(ConsumerDefinition {
                    name: queue.name,
                    queue: queue.name,
                    msg_type,
                    retries,
                    with_dlq: queue.with_dlq,
                    dlq_name: queue.dlq_name,
                    with_retry: queue.with_retry,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ExchangeKind::map(kind), lapin::ExchangeKind::Direct);
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
    fn test_consumer_definition() {
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
        let topology = AmqpTopology::new();

        let exch_def = ExchangeDefinition::name("exchange");
        let topology = topology.exchange(exch_def.clone());
        assert_eq!(topology.exchanges[0].name, exch_def.name);

        let queue_def = QueueDefinition::name("queue");
        let topology = topology.queue(queue_def.clone());
        assert_eq!(topology.queues[0].name, queue_def.name);

        let consumer_def = topology.consumer_def("queue", "msg_type");
        assert!(consumer_def.is_some());

        let queue_def = QueueDefinition::name("queue").with_retry(1000, 3);
        let topology = topology.queue(queue_def.clone());
        assert!(topology.consumer_def("queue", "msg_type").is_some());
    }
}
