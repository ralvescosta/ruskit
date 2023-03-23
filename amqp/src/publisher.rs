use lapin::Channel;
use opentelemetry::Context;
use std::sync::Arc;

pub trait Publisher {
    fn simple_publish<T>(&self, ctx: &Context, target: &str, msg: T) -> Result<(), ()>;
}

pub struct AmqpPublisher {
    channel: Arc<Channel>,
}
