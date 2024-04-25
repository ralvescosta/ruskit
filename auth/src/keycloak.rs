use super::{errors::AuthError, manager::JwtManager, types::TokenClaims};
use async_trait::async_trait;
use configs::IdentityServerConfigs;
use jsonwebtoken::jwk::JwkSet;
use moka::future::{Cache, CacheBuilder};
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span, Status, Tracer},
    Context, KeyValue,
};
use std::{borrow::Cow, sync::Arc, time::Duration};
use tracing::error;

pub struct KeycloakJwtManager {
    jwks_cache: Cache<String, JwkSet>,
    jwt_cache: Cache<String, TokenClaims>,
    tracer: BoxedTracer,
    cfg: IdentityServerConfigs,
}

impl KeycloakJwtManager {
    pub fn new(cfg: &IdentityServerConfigs) -> Arc<Self> {
        Arc::new(Self {
            jwks_cache: CacheBuilder::new(2)
                .time_to_live(Duration::from_millis(86_400_000))
                .time_to_idle(Duration::from_millis(3_600_000))
                .build(),
            jwt_cache: CacheBuilder::new(10_000)
                .time_to_live(Duration::from_millis(86_400_000))
                .time_to_idle(Duration::from_millis(1_800_000))
                .build(),
            tracer: global::tracer("keycloak-jwt-manager"),
            cfg: cfg.to_owned(),
        })
    }
}

#[async_trait]
impl JwtManager for KeycloakJwtManager {
    async fn verify(&self, ctx: &Context, token: &str) -> Result<TokenClaims, AuthError> {
        let mut span = self.tracer.start_with_context("authenticate", ctx);

        if let Some(cached_claim) = self.jwt_cache.get(token).await {
            span.set_attribute(KeyValue::new("jwt-cached", true));
            span.set_status(Status::Ok);
            return Ok(cached_claim);
        };

        span.set_attribute(KeyValue::new("jwt-cached", false));

        let jwks: JwkSet = match self.jwks_cache.get("jwks").await {
            Some(jwks) => {
                span.set_attribute(KeyValue::new("jwks-cached", true));
                jwks
            }
            _ => {
                span.set_attribute(KeyValue::new("jwks-cached", false));
                let jwks = self.get_jwks(&mut span).await?;
                self.jwks_cache.insert("jwks".into(), jwks.clone()).await;
                jwks
            }
        };

        let decoded_token =
            self.decode_token(token, &jwks, &self.cfg.audience, &self.cfg.issuer)?;

        span.set_status(Status::Ok);

        let claim = TokenClaims::from_keycloak(&decoded_token.claims)?;

        self.jwt_cache.insert(token.into(), claim.clone()).await;

        Ok(claim)
    }
}

impl KeycloakJwtManager {
    async fn get_jwks(&self, span: &mut BoxedSpan) -> Result<JwkSet, AuthError> {
        // http://BASE_URL/realms/proteu/protocol/openid-connect/certs
        let endpoint = format!(
            "{}/realms/{}/protocol/openid-connect/certs",
            self.cfg.url, self.cfg.realm,
        );

        let res = match reqwest::get(&endpoint).await {
            Err(err) => {
                error!(error = err.to_string(), "error to get jwks from auth0 api");
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to get jwks from auth0 api"),
                });
                Err(AuthError::CouldNotRetrieveJWKS)
            }
            Ok(r) => Ok(r),
        }?;

        let val = match res.json::<JwkSet>().await {
            Err(err) => {
                error!(error = err.to_string(), "error deserializing the jwks");
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error deserializing the jwks"),
                });
                Err(AuthError::CouldNotRetrieveJWKS)
            }
            Ok(v) => Ok(v),
        }?;

        Ok(val)
    }
}
