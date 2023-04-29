use std::borrow::Cow;

use crate::viewmodels::HTTPError;
use actix_http::StatusCode;
use opentelemetry::{
    trace::{Status, TraceContextExt},
    Context,
};
use serde_json::{Map, Value};
use tracing::error;
use validator::{Validate, ValidationErrors, ValidationErrorsKind};

pub fn body_validator<T>(ctx: &Context, body: &T) -> Result<(), HTTPError>
where
    T: Validate,
{
    let span = ctx.span();

    match body.validate() {
        Err(err) => {
            error!("request with unformatted body");

            span.set_status(Status::Error {
                description: Cow::from("request with unformatted body"),
            });

            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.as_u16(),
                message: "unformatted body".to_owned(),
                details: Value::Object(Map::from_iter(flatten_errors(&err))),
            })
        }
        _ => Ok(()),
    }
}

fn flatten_errors(errors: &ValidationErrors) -> Vec<(String, Value)> {
    _flatten_errors(errors, None, None)
}

fn _flatten_errors(
    errors: &ValidationErrors,
    path: Option<String>,
    indent: Option<u16>,
) -> Vec<(String, Value)> {
    errors
        .errors()
        .iter()
        .flat_map(|(&field, err)| {
            let indent = indent.unwrap_or(0);
            let actual_path = path
                .as_ref()
                .map(|path| [path.as_str(), field].join("."))
                .unwrap_or_else(|| field.to_owned());
            match err {
                ValidationErrorsKind::Field(field_errors) => field_errors
                    .iter()
                    .map(|error| (actual_path.clone(), Value::String(error.to_string())))
                    .collect::<Vec<_>>(),
                ValidationErrorsKind::List(list_error) => list_error
                    .iter()
                    .flat_map(|(index, errors)| {
                        let actual_path = format!("{}[{}]", actual_path.as_str(), index);
                        _flatten_errors(errors, Some(actual_path), Some(indent + 1))
                    })
                    .collect::<Vec<_>>(),
                ValidationErrorsKind::Struct(struct_errors) => {
                    _flatten_errors(struct_errors, Some(actual_path), Some(indent + 1))
                }
            }
        })
        .collect::<Vec<_>>()
}
