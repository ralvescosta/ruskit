use amqp::{
    channel::new_amqp_channel,
    dispatcher::{AmqpDispatcher, ConsumerHandler, Dispatcher},
    errors::AmqpError,
    exchange,
    publisher::{AmqpPublisher, Publisher},
    queue,
    topology::{AmqpTopology, Topology},
};
use async_trait::async_trait;
use env::{Configs, Empty};
use opentelemetry::Context;
use serde::Serialize;
use std::{borrow::Cow, sync::Arc};

#[derive(Debug, Default, Serialize)]
struct Message {}

struct MyHandler {}

#[async_trait]
impl ConsumerHandler for MyHandler {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let cfg = Configs::<Empty>::default();
    let channel = new_amqp_channel(&cfg).await.unwrap();

    AmqpTopology::new(channel.clone())
        .exchange(&exchange::ExchangeDefinition::new("oi"))
        .queue(&queue::QueueDefinition::new("oi"))
        .queue_binding(&queue::QueueBinding::new("queue").exchange("exchange"))
        .install()
        .await
        .expect("");

    let publisher = AmqpPublisher::new(channel.clone());
    publisher
        .simple_publish(&Context::default(), "target", &Message {}, None)
        .await
        .expect("");

    let dispatcher = AmqpDispatcher::new(channel)
        .register("queue", &Message::default(), Arc::new(MyHandler {}))
        .consume_blocking()
        .await;
}
