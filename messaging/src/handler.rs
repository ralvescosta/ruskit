use async_trait::async_trait;
use opentelemetry::Context;
use std::{collections::HashMap, error::Error};

pub struct ConsumerPayload {
    pub from: String,
    pub msg_type: String,
    pub payload: Box<[u8]>,
    pub headers: HashMap<String, String>,
}

#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    async fn exec(&self, ctx: &Context, payload: &ConsumerPayload) -> Result<(), Box<dyn Error>>;
}
