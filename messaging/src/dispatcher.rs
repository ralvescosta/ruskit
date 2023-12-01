use crate::{errors::MessagingError, handler::ConsumerHandler};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DispatcherDefinition {
    pub name: String,
    pub msg_type: String,
}

#[async_trait]
pub trait Dispatcher: Send + Sync {
    fn register(self, definition: &DispatcherDefinition, handler: Arc<dyn ConsumerHandler>)
        -> Self;

    async fn consume_blocking(&self) -> Result<(), MessagingError>;
}
