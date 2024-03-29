#[derive(Debug, Clone, Default)]
pub struct QueueDefinition {
    pub(crate) name: String,
    pub(crate) durable: bool,
    pub(crate) delete: bool,
    pub(crate) exclusive: bool,
    pub(crate) passive: bool,
    pub(crate) no_wait: bool,
    pub(crate) ttl: Option<i32>,
    pub(crate) dlq_name: Option<String>,
    pub(crate) retry_name: Option<String>,
    pub(crate) retry_ttl: Option<i32>,
    pub(crate) retries: Option<i32>,
}

impl QueueDefinition {
    pub fn new(name: &str) -> QueueDefinition {
        QueueDefinition {
            name: name.to_owned(),
            durable: false,
            delete: false,
            exclusive: false,
            passive: false,
            no_wait: false,
            ttl: None,
            dlq_name: None,
            retry_name: None,
            retry_ttl: None,
            retries: None,
        }
    }

    pub fn durable(mut self) -> Self {
        self.durable = true;
        self
    }

    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    pub fn exclusive(mut self) -> Self {
        self.exclusive = true;
        self
    }

    pub fn ttl(mut self, ttl: i32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_dlq(mut self) -> Self {
        self.dlq_name = Some(format!("{}-dlq", self.name));
        self
    }

    pub fn with_retry(mut self, ttl: i32, retries: i32) -> Self {
        self.retry_name = Some(format!("{}-retry", self.name));
        self.retries = Some(retries);
        self.retry_ttl = Some(ttl);
        self
    }
}

pub struct QueueBinding<'qeb> {
    pub(crate) queue_name: &'qeb str,
    pub(crate) exchange_name: &'qeb str,
    pub(crate) routing_key: &'qeb str,
}

impl<'qeb> QueueBinding<'qeb> {
    pub fn new(queue: &'qeb str) -> QueueBinding<'qeb> {
        QueueBinding {
            queue_name: queue,
            exchange_name: "",
            routing_key: "",
        }
    }

    pub fn exchange(mut self, exchange: &'qeb str) -> Self {
        self.exchange_name = exchange;
        self
    }

    pub fn routing_key(mut self, key: &'qeb str) -> Self {
        self.routing_key = key;
        self
    }
}
