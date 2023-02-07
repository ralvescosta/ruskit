use crate::{defs::Scopes, types::AuthMiddleware};
use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use async_trait::async_trait;
use env::{Configs, Empty};
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span, Status, Tracer},
    Context,
};
use std::{
    borrow::Cow,
    time::{Duration, SystemTime},
};
use tokio::sync::Mutex;
use tracing::error;

pub struct Auth0Middleware {
    jwks: Mutex<Option<JWKS>>,
    jwks_retrieved_at: SystemTime,
    authority: String,
    tracer: BoxedTracer,
}

impl Auth0Middleware {
    pub fn new(cfg: &Configs<Empty>) -> Auth0Middleware {
        Auth0Middleware {
            jwks: Mutex::new(None),
            jwks_retrieved_at: SystemTime::now(),
            authority: String::new(), //get from the config
            tracer: global::tracer("auth0_middleware"),
        }
    }
}

#[async_trait]
impl AuthMiddleware for Auth0Middleware {
    async fn authenticate(&self, ctx: &Context, token: &str) -> Result<bool, ()> {
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
            Validation::Issuer(self.authority.clone()),
            Validation::SubjectPresent,
        ];

        let jwk = jwks.find(&kid).expect("Specified key not found in set");
        let res = validate(token, jwk, validations);

        Ok(res.is_ok())
    }

    async fn authorize(
        &self,
        ctx: &Context,
        token: &str,
        required_scope: Scopes,
    ) -> Result<bool, ()> {
        let mut span = self.tracer.start_with_context("authenticate", ctx);

        let jwks = self.retrieve_jwks(&mut span).await?;

        Ok(true)
    }
}

impl Auth0Middleware {
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
        let res = match reqwest::get("").await {
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
}
