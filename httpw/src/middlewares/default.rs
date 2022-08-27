use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

pub async fn handler() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(ContentType::plaintext())
        .body("Not Found")
}