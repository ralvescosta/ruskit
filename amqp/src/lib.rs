pub mod channel;
pub mod errors;
pub mod exchange;
pub mod message;
pub mod otel;
pub mod publisher;
pub mod queue;
pub mod topology;

use channel::new_amqp_channel;
use env::{Configs, Empty};
use topology::{AmqpTopology, Topology};

async fn oi() {
    let cfg = Configs::<Empty>::default();
    let channel = new_amqp_channel(&cfg).await.unwrap();

    AmqpTopology::new(channel)
        .exchange(&exchange::ExchangeDefinition::new("oi"))
        .queue(&queue::QueueDefinition::new("oi"))
        .queue_binding(&queue::QueueBinding::new())
        .install()
        .await
        .expect("");
}
