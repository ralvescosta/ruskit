use crate::{errors::MessagingError, handler::ConsumerHandler};
use async_trait::async_trait;
use std::sync::Arc;

#[cfg(feature = "mocks")]
use mockall::*;

#[derive(Debug, Clone)]
pub struct DispatcherDefinition {
    pub name: String,
    pub msg_type: String,
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait Dispatcher: Send + Sync {
    fn register(self, definition: &DispatcherDefinition, handler: Arc<dyn ConsumerHandler>)
        -> Self;

    async fn consume_blocking(&self) -> Result<(), MessagingError>;
}
