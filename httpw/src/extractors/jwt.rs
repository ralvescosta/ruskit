use crate::viewmodels::HttpErrorViewModel;
use actix_web::error::{ErrorUnauthorized, ErrorInternalServerError};
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, FromRequest, HttpRequest};
use auth::jwt_manager::{JwtManager};
use opentelemetry::Context;
use serde::{Deserialize, Serialize};
use std::future::{Future};
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub struct JwtAuthenticateExtractor {
    pub user_id: String,
}

impl FromRequest for JwtAuthenticateExtractor {
    type Error = ActixWebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|h| h.to_str().unwrap().split_at(7).1.to_string());

        let Some(token) = token else {
            return Box::pin(async move {
                let json_error = HttpErrorViewModel {
                    status_code: http::StatusCode::UNAUTHORIZED.as_u16(),
                    message: "unauthorized".to_owned(),
                    details: "You are not logged in, please provide token".to_string(),
                };
                    
                return Err(ErrorUnauthorized(json_error));
            });
        };

        let Some(jwt_manager) = req.app_data::<Arc<dyn JwtManager + Send + Sync>>() else {
            return Box::pin(async move {
                let json_error = HttpErrorViewModel {
                    status_code: http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    message: "jwt manager internal error".to_owned(),
                    details: "no jwt manager was provided".to_string(),
                };
                
                return Err(ErrorInternalServerError(json_error));
            });
        };

        let jwt_manager = jwt_manager.clone();

        Box::pin(async move {
            let Ok(claims) = jwt_manager.verify(&Context::new(), &token).await else {
                let json_error = HttpErrorViewModel {
                    status_code: http::StatusCode::UNAUTHORIZED.as_u16(),
                    message: "unauthorized".to_owned(),
                    details: "You are not logged in, please provide token".to_string(),
                };
            
                return Err(ErrorUnauthorized(json_error));
            };

            // let user_id = claims.sub;
            // req.extensions_mut().insert::<String>(user_id.to_owned());

            Ok(JwtAuthenticateExtractor { user_id: String::new() })
        })
    }
}
