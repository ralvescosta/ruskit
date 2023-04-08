use super::{JwtManager, TokenClaims};
use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use async_trait::async_trait;
use configs::Auth0Configs;
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span, Status, Tracer},
    Context,
};
use serde_json::Value;
use std::{
    borrow::Cow,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::Mutex;
use tracing::error;

pub struct Auth0JwtManager {
    jwks: Mutex<Option<JWKS>>,
    jwks_retrieved_at: SystemTime,
    tracer: BoxedTracer,
    cfg: Auth0Configs,
}

impl Auth0JwtManager {
    pub fn new(cfg: &Auth0Configs) -> Arc<Auth0JwtManager> {
        Arc::new(Auth0JwtManager {
            jwks: Mutex::new(None),
            jwks_retrieved_at: SystemTime::now(),
            tracer: global::tracer("auth0_middleware"),
            cfg: cfg.to_owned(),
        })
    }
}

#[async_trait]
impl JwtManager for Auth0JwtManager {
    async fn verify(&self, ctx: &Context, token: &str) -> Result<TokenClaims, ()> {
        let mut span = self.tracer.start_with_context("authenticate", ctx);

        let jwks = self.retrieve_jwks(&mut span).await?;

        let kid = match token_kid(&token) {
            Ok(res) => {
                if res.is_none() {
                    error!("token with no kid");
                    span.set_status(Status::Error {
                        description: Cow::from("token with no kid"),
                    });
                    return Err(());
                }

                Ok(res.unwrap())
            }
            Err(err) => {
                error!(error = err.to_string(), "error retrieving the token kid");
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error retrieving the token kid"),
                });
                Err(())
            }
        }?;

        let validations = vec![
            Validation::Issuer(self.cfg.issuer.clone()),
            Validation::SubjectPresent,
        ];

        let jwk = match jwks.find(&kid) {
            Some(jwk) => Ok(jwk),
            _ => {
                error!("specified jwk key was not founded in set");
                span.set_status(Status::Error {
                    description: Cow::from("specified jwk key was not founded in set"),
                });
                Err(())
            }
        }?;

        let claims = match validate(token, jwk, validations) {
            Ok(res) => Ok(res.claims),
            Err(err) => {
                error!(error = err.to_string(), "invalid jwt token");
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("invalid jwt token"),
                });
                Err(())
            }
        }?;

        span.set_status(Status::Ok);

        Ok(TokenClaims {
            iss: self.get_claim_as_string("iss", &claims, &mut span)?,
            sub: self.get_claim_as_string("sub", &claims, &mut span)?,
            aud: self.get_claim_as_vec("aud", &claims, &mut span)?,
            iat: self.get_claim_as_u64("iat", &claims, &mut span)?,
            exp: self.get_claim_as_u64("exp", &claims, &mut span)?,
            scope: self.get_claim_as_string("scope", &claims, &mut span)?,
        })
    }
}

impl Auth0JwtManager {
    async fn retrieve_jwks(&self, span: &mut BoxedSpan) -> Result<JWKS, ()> {
        let mut jwks = self.jwks.lock().await;

        if jwks.is_none() {
            let new = self.get_jwks(span).await?;
            *jwks = Some(new.clone());
            return Ok(new);
        }

        let duration = match SystemTime::now().duration_since(self.jwks_retrieved_at.clone()) {
            Ok(d) => Ok(d),
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error comparing the jwks caching time"
                );
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error comparing the jwks caching time"),
                });
                Err(())
            }
        }?;

        if duration.cmp(&Duration::new(3600, 0)).is_ge() {
            let new = self.get_jwks(span).await?;
            *jwks = Some(new.clone());
            return Ok(new);
        }

        Ok(jwks.clone().unwrap())
    }

    async fn get_jwks(&self, span: &mut BoxedSpan) -> Result<JWKS, ()> {
        let res = match reqwest::get(&format!(
            "https://{}/{}",
            self.cfg.domain, ".well-known/jwks.json"
        ))
        .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error to get jwks from auth0 api");
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to get jwks from auth0 api"),
                });
                Err(())
            }
            Ok(r) => Ok(r),
        }?;

        let val = match res.json::<JWKS>().await {
            Err(err) => {
                error!(error = err.to_string(), "error deserializing the jwks");
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error deserializing the jwks"),
                });
                Err(())
            }
            Ok(v) => Ok(v),
        }?;

        return Ok(val);
    }

    fn get_claim_as_string(
        &self,
        key: &str,
        claims: &Value,
        span: &mut BoxedSpan,
    ) -> Result<String, ()> {
        match claims.get(key) {
            Some(fv) => match fv.as_str() {
                Some(value) => Ok(value.to_owned()),
                _ => {
                    error!(claim = key, "invalid jwt claim");
                    span.set_status(Status::Error {
                        description: Cow::from("invalid jwt claim"),
                    });
                    Err(())
                }
            },
            _ => {
                error!(claim = key, "invalid jwt claim");
                span.set_status(Status::Error {
                    description: Cow::from("invalid jwt claim"),
                });
                Err(())
            }
        }
    }

    fn get_claim_as_u64(&self, key: &str, claims: &Value, span: &mut BoxedSpan) -> Result<u64, ()> {
        match claims.get(key) {
            Some(fv) => match fv.as_u64() {
                Some(value) => Ok(value),
                _ => {
                    error!(claim = key, "invalid jwt claim");
                    span.set_status(Status::Error {
                        description: Cow::from("invalid jwt claim"),
                    });
                    Err(())
                }
            },
            _ => {
                error!(claim = key, "invalid jwt claim");
                span.set_status(Status::Error {
                    description: Cow::from("invalid jwt claim"),
                });
                Err(())
            }
        }
    }

    fn get_claim_as_vec(
        &self,
        key: &str,
        claims: &Value,
        span: &mut BoxedSpan,
    ) -> Result<Vec<String>, ()> {
        match claims.get(key) {
            Some(fv) => match fv.as_array() {
                Some(value) => Ok(value
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect::<Vec<String>>()),
                _ => {
                    error!(claim = key, "invalid jwt claim");
                    span.set_status(Status::Error {
                        description: Cow::from("invalid jwt claim"),
                    });
                    Err(())
                }
            },
            _ => {
                error!(claim = key, "invalid jwt claim");
                span.set_status(Status::Error {
                    description: Cow::from("invalid jwt claim"),
                });
                Err(())
            }
        }
    }
}
