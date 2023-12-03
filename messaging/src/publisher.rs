use crate::errors::MessagingError;
use async_trait::async_trait;
use opentelemetry::Context;
use std::collections::HashMap;

#[derive(Clone)]
pub enum HeaderValues {
    String(String),
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

#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(&self, ctx: &Context, infos: &PublishInfos) -> Result<(), MessagingError>;
}
