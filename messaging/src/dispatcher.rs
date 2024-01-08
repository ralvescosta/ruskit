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

impl DispatcherDefinition {
    pub fn new<T>(name: T, msg_type: T) -> Self
    where
        T: Into<String>,
    {
        DispatcherDefinition {
            name: name.into(),
            msg_type: msg_type.into(),
        }
    }
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait Dispatcher: Send + Sync {
    fn register(self, definition: &DispatcherDefinition, handler: Arc<dyn ConsumerHandler>)
        -> Self;

    async fn consume_blocking(&self) -> Result<(), MessagingError>;
}
