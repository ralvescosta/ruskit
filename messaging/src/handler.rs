use crate::errors::MessagingError;
use async_trait::async_trait;
use opentelemetry::Context;
use std::collections::HashMap;

#[cfg(feature = "mocks")]
use mockall::*;

#[derive(Clone, Default)]
pub struct ConsumerMessage {
    pub from: String,
    pub msg_type: String,
    pub data: Box<[u8]>,
    pub headers: Option<HashMap<String, String>>,
}

impl ConsumerMessage {
    pub fn new<T>(
        from: T,
        msg_type: T,
        data: &[u8],
        headers: Option<HashMap<String, String>>,
    ) -> Self
    where
        T: Into<String>,
    {
        ConsumerMessage {
            from: from.into(),
            msg_type: msg_type.into(),
            data: data.into(),
            headers,
        }
    }
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    async fn exec(&self, ctx: &Context, msg: &ConsumerMessage) -> Result<(), MessagingError>;
}
