use crate::HealthReadinessService;
use actix_web::{error::HttpError, get, http::StatusCode, web, HttpResponse};
use http_components::viewmodels::HTTPError;
use std::sync::Arc;

#[get("/health")]
pub async fn health_handler(
    service: web::Data<Arc<dyn HealthReadinessService>>,
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
