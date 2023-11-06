use crate::viewmodels::HTTPError;
use actix_web::{error::HttpError, get, web, HttpResponse};
use health_readiness::HealthReadinessService;
use std::sync::Arc;

#[get("/health")]
pub async fn health_handler(
    service: web::Data<Arc<dyn HealthReadinessService>>,
) -> Result<HttpResponse, HttpError> {
    match service.validate().await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(HTTPError::internal_server_error(
                e.to_string(),
                e.to_string(),
            )),
        ),
    }
}
