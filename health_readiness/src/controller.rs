use std::sync::Arc;

use actix_web::{error::HttpError, get, http::StatusCode, web, HttpResponse};
use httpw::viewmodels::HTTPError;

use crate::HealthReadinessService;

#[get("/health")]
pub(crate) async fn health_handler(
    service: web::Data<Arc<dyn HealthReadinessService + Send + Sync>>,
) -> Result<HttpResponse, HttpError> {
    match service.validate().await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().json(HTTPError {
            details: e.to_string(),
            message: e.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        })),
    }
}
