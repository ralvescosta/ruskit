use async_trait::async_trait;
use opentelemetry::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Default, Serialize, Deserialize)]
pub struct Session(pub Map<String, Value>);

#[derive(Default, Serialize, Deserialize)]
pub struct TokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub iat: u64,
    pub exp: u64,
    pub scope: String,
    pub session: Session,
}

#[async_trait]
pub trait JwtManager: Send + Sync {
    async fn verify(&self, ctx: &Context, token: &str) -> Result<TokenClaims, ()>;
}
