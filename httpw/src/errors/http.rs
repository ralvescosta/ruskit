use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("bad request")]
    BadRequest,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("conflict")]
    Conflict,

    #[error("internal error")]
    InternalError,

    #[error("timeout")]
    Timeout,
}

impl ResponseError for HttpError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            HttpError::BadRequest => StatusCode::BAD_REQUEST,
            HttpError::Unauthorized => StatusCode::UNAUTHORIZED,
            HttpError::Forbidden => StatusCode::FORBIDDEN,
            HttpError::NotFound => StatusCode::NOT_FOUND,
            HttpError::Conflict => StatusCode::CONFLICT,
            HttpError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}
