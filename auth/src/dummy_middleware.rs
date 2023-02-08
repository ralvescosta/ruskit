use crate::{AuthMiddleware, Scopes};
use async_trait::async_trait;
use opentelemetry::Context;
use std::sync::Arc;

pub struct DummyMiddleware;

impl DummyMiddleware {
    pub fn new() -> Arc<DummyMiddleware> {
        Arc::new(DummyMiddleware {})
    }
}

#[async_trait]
impl AuthMiddleware for DummyMiddleware {
    async fn authenticate(&self, _ctx: &Context, _token: &str) -> Result<bool, ()> {
        Ok(true)
    }

    async fn authorize(
        &self,
        _ctx: &Context,
        _token: &str,
        _required_scope: Scopes,
    ) -> Result<bool, ()> {
        Ok(true)
    }
}
