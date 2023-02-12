use crate::viewmodels::HttpErrorViewModel;
use actix_web::{http::StatusCode, web, Responder};

pub async fn not_found() -> impl Responder {
    web::Json(HttpErrorViewModel {
        status_code: StatusCode::NOT_FOUND.as_u16(),
        message: "not found".to_owned(),
        details: "the resource was not founded".to_owned(),
    })
}
