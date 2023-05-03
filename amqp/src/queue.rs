/// `QueueDefinition` represents a queue definition for RabbitMQ.
///
/// # Examples
///
/// ```rust,no_run
/// use ruskit::amqp::queue::QueueDefinition;
///
/// fn main() {
///     // Define a new queue named "example_queue" durable with a TTL of 10 seconds.
///     let queue_def = QueueDefinition::new("example_queue").durable().ttl(10);
/// }
/// ```
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
    /// Creates a new `QueueDefinition` with the given name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Define a new queue named "example_queue".
    ///     let queue_def = QueueDefinition::new("example_queue");
    /// }
    /// ```
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

    /// Marks the queue as durable.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Define a new durable queue named "example_queue".
    ///     let queue_def = QueueDefinition::new("example_queue").durable();
    /// }
    /// ```
    pub fn durable(mut self) -> Self {
        self.durable = true;
        self
    }

    /// Marks the queue as eligible for auto-deletion.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Define a new queue named "example_queue" that will be deleted automatically.
    ///     let queue_def = QueueDefinition::new("example_queue").delete();
    /// }
    /// ```
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    /// Marks the queue as eligible for exclusive.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Define a new exclusive queue named "example_queue".
    ///     let queue_def = QueueDefinition::new("example_queue").exclusive();
    /// }
    /// ```
    pub fn exclusive(mut self) -> Self {
        self.exclusive = true;
        self
    }

    /// Sets the queue's TTL (time-to-live) in seconds.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Define a new queue named "example_queue" with a TTL of 10 seconds.
    ///     let queue_def = QueueDefinition::new("example_queue").exclusive();
    /// }
    /// ```
    pub fn ttl(mut self, ttl: i32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Adds a dead-letter queue to the queue definition.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Add a DLQ queue to the queue named "example_queue".
    ///     let queue_def = QueueDefinition::new("example_queue").with_dlq();
    /// }
    /// ```
    pub fn with_dlq(mut self) -> Self {
        self.dlq_name = Some(format!("{}-dlq", self.name));
        self
    }

    /// Adds a retry strategy to the queue definition with TTL in seconds.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruskit::amqp::queue::QueueDefinition;
    ///
    /// fn main() {
    ///     // Define a new queue named "example_queue" with a TTL retry of 10 seconds and retry count of 3 times.
    ///     let queue_def = QueueDefinition::new("example_queue").with_retry(10, 3);
    /// }
    /// ```
    pub fn with_retry(mut self, ttl: i32, retries: i32) -> Self {
        self.retry_name = Some(format!("{}-retry", self.name));
        self.retries = Some(retries);
        self.retry_ttl = Some(ttl);
        self
    }
}

/// Represents a binding between a queue and an exchange in RabbitMQ.
///
/// A queue binding specifies how messages are routed from an exchange to a
/// queue based on a routing key. An instance of this struct represents a
/// binding between a specific queue and exchange, and allows configuring the
/// routing key used for message delivery.
///
/// # Examples
///
/// ```rust,no_run
/// use ruskit::amqp::queue::QueueBinding;
///
/// fn main() {
///     // Create a new binding with the default routing key.
///     let binding = QueueBinding::new("my_queue");
///
///     // Update the exchange and routing key of the binding.
///     let updated_binding = binding
///         .exchange("my_exchange")
///         .routing_key("my_routing_key");
/// }
///
/// ```
pub struct QueueBinding<'qeb> {
    /// The name of the queue to which this binding applies.
    pub(crate) queue_name: &'qeb str,
    /// The name of the exchange to which this binding applies.
    pub(crate) exchange_name: &'qeb str,
    /// The routing key used for message delivery from the exchange to the queue.
    pub(crate) routing_key: &'qeb str,
}

impl<'qeb> QueueBinding<'qeb> {
    /// Creates a new binding for the given queue, with default values for the
    /// exchange name and routing key.
    pub fn new(queue: &'qeb str) -> QueueBinding<'qeb> {
        QueueBinding {
            queue_name: queue,
            exchange_name: "",
            routing_key: "",
        }
    }

    /// Sets the exchange name for this binding, and returns a new instance of
    /// `QueueBinding` with the updated value.
    pub fn exchange(mut self, exchange: &'qeb str) -> Self {
        self.exchange_name = exchange;
        self
    }

    /// Sets the routing key for this binding, and returns a new instance of
    /// `QueueBinding` with the updated value.
    pub fn routing_key(mut self, key: &'qeb str) -> Self {
        self.routing_key = key;
        self
    }
}
