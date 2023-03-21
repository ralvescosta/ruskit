use crate::{
    errors::AmqpError,
    exchange::{ExchangeBinding, ExchangeDefinition},
    queue::{QueueBinding, QueueDefinition},
};
use lapin::Channel;
use std::{collections::HashMap, sync::Arc};

pub trait Topology<'tp> {
    fn exchange(self, def: &'tp ExchangeDefinition) -> Self;
    fn queue(self, def: &'tp QueueDefinition) -> Self;
    fn exchange_binding(self, binding: &'tp ExchangeBinding) -> Self;
    fn queue_binding(self, binding: &'tp QueueBinding) -> Self;
    fn install(&self) -> Result<(), AmqpError>;
}

pub struct AmqpTopology<'tp> {
    pub(crate) channel: Arc<Channel>,
    pub(crate) queues: HashMap<&'tp str, &'tp QueueDefinition<'tp>>,
    pub(crate) queues_binding: HashMap<&'tp str, &'tp QueueBinding<'tp>>,
    pub(crate) exchanges: Vec<&'tp ExchangeDefinition<'tp>>,
    pub(crate) exchanges_binding: Vec<&'tp ExchangeBinding>,
}

impl<'tp> AmqpTopology<'tp> {
    pub fn new(channel: Arc<Channel>) -> AmqpTopology<'tp> {
        AmqpTopology {
            channel,
            queues: HashMap::default(),
            queues_binding: HashMap::default(),
            exchanges: vec![],
            exchanges_binding: vec![],
        }
    }
}

impl<'tp> Topology<'tp> for AmqpTopology<'tp> {
    fn exchange(mut self, def: &'tp ExchangeDefinition) -> Self {
        self.exchanges.push(def);
        self
    }

    fn queue(mut self, def: &'tp QueueDefinition) -> Self {
        self.queues.insert(def.name, def);
        self
    }

    fn exchange_binding(mut self, binding: &'tp ExchangeBinding) -> Self {
        self.exchanges_binding.push(binding);
        self
    }

    fn queue_binding(mut self, binding: &'tp QueueBinding) -> Self {
        self.queues_binding.insert(binding.queue_name, binding);
        self
    }

    fn install(&self) -> Result<(), AmqpError> {
        Ok(())
    }
}

impl<'tp> AmqpTopology<'tp> {
    fn install_exchange(&self) {}

    fn install_queue(&self) {}
}
