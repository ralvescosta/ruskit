use actix_cors::Cors;
use actix_web::http;

pub fn config() -> Cors {
    Cors::default()
        .allowed_origin("All")
        .send_wildcard()
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::ORIGIN,
            http::header::LOCATION,
            http::header::HOST,
            http::header::USER_AGENT,
            http::header::CONTENT_LENGTH,
            http::header::CONTENT_TYPE,
        ])
        .max_age(3600)
}
