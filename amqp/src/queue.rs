#[derive(Debug, Clone)]
pub struct QueueDefinition {
    pub(crate) name: &'static str,
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
    pub fn new(name: &'static str) -> QueueDefinition {
        QueueDefinition {
            name,
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
    pub fn new() -> QueueBinding<'qeb> {
        QueueBinding {
            queue_name: "",
            exchange_name: "",
            routing_key: "",
        }
    }
}
