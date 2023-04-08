use crate::viewmodels::HTTPError;
use actix_web::{
    error::{InternalError, JsonPayloadError},
    web::JsonConfig,
    HttpRequest, HttpResponse,
};

pub fn handler() -> JsonConfig {
    JsonConfig::default().error_handler(|err: JsonPayloadError, _req: &HttpRequest| {
        InternalError::from_response(
            format!("JSON error: {:?}", err),
            HttpResponse::BadRequest().json(HTTPError {
                status_code: 400,
                message: String::from("wrong body format"),
                details: format!("{}", err),
            }),
        )
        .into()
    })
}
