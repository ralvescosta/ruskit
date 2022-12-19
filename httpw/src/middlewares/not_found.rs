use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

pub async fn not_found() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(ContentType::plaintext())
        .body("Not Found")
}
