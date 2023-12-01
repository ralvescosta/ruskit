use async_trait::async_trait;
use opentelemetry::Context;
use std::error::Error;

#[async_trait]
pub trait ConsumerHandler<T>: Send + Sync {
    async fn exec(&self, ctx: &Context, payload: &T) -> Result<(), Box<dyn Error>>;
}
