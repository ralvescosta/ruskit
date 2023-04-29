use crate::viewmodels::HTTPError;
use actix_web::{HttpResponse, Responder};

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(HTTPError::not_found(
        "not found",
        "the resource was not founded",
    ))
}
