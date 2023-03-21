pub mod channel;
pub mod errors;
pub mod exchange;
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
        .exchange(&exchange::ExchangeDefinition { name: "oi" })
        .queue(&queue::QueueDefinition { name: "oi" })
        .queue_binding(&queue::QueueBinding { queue_name: "oi" })
        .install();
}
