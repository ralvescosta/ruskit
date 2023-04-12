use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HTTPError {
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

impl ResponseError for HTTPError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            HTTPError::BadRequest => StatusCode::BAD_REQUEST,
            HTTPError::Unauthorized => StatusCode::UNAUTHORIZED,
            HTTPError::Forbidden => StatusCode::FORBIDDEN,
            HTTPError::NotFound => StatusCode::NOT_FOUND,
            HTTPError::Conflict => StatusCode::CONFLICT,
            HTTPError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            HTTPError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}
