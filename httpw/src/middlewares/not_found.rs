use crate::viewmodels::HTTPError;
use actix_web::{http::StatusCode, HttpResponse, Responder};

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(HTTPError {
        status_code: StatusCode::NOT_FOUND.as_u16(),
        message: "not found".to_owned(),
        details: "the resource was not founded".to_owned(),
    })
}
