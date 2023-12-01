use async_trait::async_trait;
use opentelemetry::Context;
use std::{collections::HashMap, error::Error};

pub struct PublishPayload {
    pub to: String,
    pub from: String,
    pub key: String,
    pub msg_type: String,
    pub payload: Box<[u8]>,
    pub headers: Option<HashMap<String, String>>,
}

#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(&self, ctx: &Context, payload: &PublishPayload) -> Result<(), Box<dyn Error>>;
}
