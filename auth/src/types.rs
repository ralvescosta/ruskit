use crate::defs::Scopes;
use async_trait::async_trait;
use opentelemetry::Context;

#[async_trait]
pub trait AuthMiddleware {
    async fn authenticate(&self, ctx: &Context, token: &str) -> Result<bool, ()>;
    async fn authorize(
        &self,
        ctx: &Context,
        token: &str,
        required_scope: Scopes,
    ) -> Result<bool, ()>;
}
