use async_trait::async_trait;
use opentelemetry::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub iat: usize,
    pub exp: usize,
    pub scope: String,
    pub permissions: Vec<String>,
}

#[async_trait]
pub trait JwtManager {
    async fn verify(&self, ctx: &Context, token: &str) -> Result<TokenClaims, ()>;
}
