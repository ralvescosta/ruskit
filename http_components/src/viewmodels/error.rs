use actix_http::StatusCode;
use actix_web::{http::header::ContentType, HttpResponse, ResponseError};
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HTTPError {
    pub status_code: u16,
    pub message: String,
    pub details: Value,
}

impl HTTPError {
    pub fn bad_request(
        message: impl Into<std::string::String>,
        details: impl Into<std::string::String>,
    ) -> HTTPError {
        HTTPError {
            status_code: StatusCode::BAD_REQUEST.as_u16(),
            message: message.into(),
            details: Value::String(details.into()),
        }
    }

    pub fn unauthorized(
        message: impl Into<std::string::String>,
        details: impl Into<std::string::String>,
    ) -> HTTPError {
        HTTPError {
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
            message: message.into(),
            details: Value::String(details.into()),
        }
    }

    pub fn forbidden(
        message: impl Into<std::string::String>,
        details: impl Into<std::string::String>,
    ) -> HTTPError {
        HTTPError {
            status_code: StatusCode::FORBIDDEN.as_u16(),
            message: message.into(),
            details: Value::String(details.into()),
        }
    }

    pub fn not_found(
        message: impl Into<std::string::String>,
        details: impl Into<std::string::String>,
    ) -> HTTPError {
        HTTPError {
            status_code: StatusCode::NOT_FOUND.as_u16(),
            message: message.into(),
            details: Value::String(details.into()),
        }
    }

    pub fn conflict(
        message: impl Into<std::string::String>,
        details: impl Into<std::string::String>,
    ) -> HTTPError {
        HTTPError {
            status_code: StatusCode::CONFLICT.as_u16(),
            message: message.into(),
            details: Value::String(details.into()),
        }
    }

    pub fn internal_server_error(
        message: impl Into<std::string::String>,
        details: impl Into<std::string::String>,
    ) -> HTTPError {
        HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: message.into(),
            details: Value::String(details.into()),
        }
    }
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
