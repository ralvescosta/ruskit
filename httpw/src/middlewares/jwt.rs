use crate::viewmodels::HttpErrorViewModel;
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, FromRequest, HttpMessage, HttpRequest};
use env::AppConfig;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub struct JwtAuthorizationMiddleware {
    pub user_id: String,
}

impl FromRequest for JwtAuthorizationMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // let cfg = req.app_data::<AppConfig>().unwrap();

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|h| h.to_str().unwrap().split_at(7).1.to_string());

        if token.is_none() {
            let json_error = HttpErrorViewModel {
                status_code: http::StatusCode::UNAUTHORIZED.as_u16(),
                message: "unauthorized".to_owned(),
                details: "You are not logged in, please provide token".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let claims = match decode::<TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret("".as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let json_error = HttpErrorViewModel {
                    status_code: http::StatusCode::UNAUTHORIZED.as_u16(),
                    message: "unauthorized".to_owned(),
                    details: "invalid token".to_string(),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = claims.sub;
        req.extensions_mut().insert::<String>(user_id.to_owned());

        ready(Ok(JwtAuthorizationMiddleware { user_id }))
    }
}
