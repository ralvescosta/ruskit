use crate::errors::MessagingError;
use async_trait::async_trait;
use opentelemetry::Context;
use std::collections::HashMap;

#[cfg(feature = "mocks")]
use mockall::*;

#[derive(Clone)]
pub enum HeaderValues {
    ShortString(String),
    LongString(String),
    Int(i8),
    LongInt(i32),
}

#[derive(Clone)]
pub struct PublishInfos {
    pub to: String,
    pub from: String,
    pub key: String,
    pub msg_type: String,
    pub payload: Box<[u8]>,
    pub headers: Option<HashMap<String, HeaderValues>>,
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(&self, ctx: &Context, infos: &PublishInfos) -> Result<(), MessagingError>;
}
