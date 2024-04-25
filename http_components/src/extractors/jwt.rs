use crate::middlewares::otel::HTTPExtractor;
use crate::viewmodels::HTTPError;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::Data;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, FromRequest, HttpRequest};
use auth::{manager::JwtManager, types::TokenClaims};
use opentelemetry::global;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct JwtAuthenticateExtractor {
    pub claims: TokenClaims,
}

impl FromRequest for JwtAuthenticateExtractor {
    type Error = ActixWebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let ctx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HTTPExtractor::new(req.headers()))
        });

        let token = match req.headers().get(http::header::AUTHORIZATION) {
            Some(header_value) => match header_value.to_str() {
                Ok(value) => match value.strip_prefix("Bearer ") {
                    Some(t) => t.to_owned(),
                    None => {
                        return unauthorized_pined("you are not logged in, please provide a token");
                    }
                },
                Err(_) => {
                    return unauthorized_pined("you are not logged in, please provide a token");
                }
            },
            None => {
                return unauthorized_pined("you are not logged in, please provide a token");
            }
        };

        let jwt_manager = match req.app_data::<Data<Arc<dyn JwtManager>>>() {
            Some(jm) => jm.clone(),
            None => return unauthorized_pined("no jwt manager was provided"),
        };

        Box::pin(async move {
            let Ok(claims) = jwt_manager.verify(&ctx, &token).await else {
                return unauthorized("invalid token");
            };

            Ok(JwtAuthenticateExtractor { claims })
        })
    }
}

fn unauthorized(details: &str) -> Result<JwtAuthenticateExtractor, actix_web::Error> {
    Err(ErrorUnauthorized(HTTPError::unauthorized(
        "unauthorized",
        details,
    )))
}

fn unauthorized_pined(
    details: &'static str,
) -> Pin<Box<dyn Future<Output = Result<JwtAuthenticateExtractor, ActixWebError>>>> {
    Box::pin(async move {
        return unauthorized(details);
    })
}
