use actix_http::StatusCode;
use actix_web::{http::header::ContentType, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct HTTPError {
    pub status_code: u16,
    pub message: String,
    pub details: String,
}

impl fmt::Display for HTTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl ResponseError for HTTPError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::try_from(self.status_code).unwrap_or_default()
    }
}
