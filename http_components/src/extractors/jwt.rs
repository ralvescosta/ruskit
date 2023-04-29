use crate::middlewares::otel::HTTPExtractor;
use crate::viewmodels::HTTPError;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::Data;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, FromRequest, HttpRequest};
use auth::jwt_manager::{JwtManager, TokenClaims};
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

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|h| h.to_str().unwrap().split_at(7).1.to_string());

        let Some(token) = token else {
            return Box::pin(async move {
                return Err(ErrorUnauthorized(
                    HTTPError::unauthorized("unauthorized", "you are not logged in, please provide token")),
                );
            });
        };

        let Some(jwt_manager) = req.app_data::<Data<Arc<dyn JwtManager>>>() else {
            return Box::pin(async move {
                return Err(ErrorInternalServerError(
                    HTTPError::internal_server_error("jwt manager internal error", "no jwt manager was provided")),
                );
            });
        };

        let jwt_manager = jwt_manager.clone();

        Box::pin(async move {
            let Ok(claims) = jwt_manager.verify(&ctx, &token).await else {
                return Err(ErrorUnauthorized(HTTPError::unauthorized("unauthorized", "invalid token")));
            };

            Ok(JwtAuthenticateExtractor { claims })
        })
    }
}
