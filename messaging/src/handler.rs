use crate::errors::MessagingError;
use async_trait::async_trait;
use opentelemetry::Context;
use std::collections::HashMap;

#[cfg(test)]
use mockall::*;
#[cfg(feature = "mocks")]
use mockall::*;

#[derive(Clone, Default)]
pub struct ConsumerPayload {
    pub from: String,
    pub msg_type: String,
    pub payload: Box<[u8]>,
    pub headers: Option<HashMap<String, String>>,
}

#[cfg_attr(test, automock)]
#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    async fn exec(&self, ctx: &Context, payload: &ConsumerPayload) -> Result<(), MessagingError>;
}
