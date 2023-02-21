use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct HttpErrorViewModel {
    pub status_code: u16,
    pub message: String,
    pub details: String,
}

impl fmt::Display for HttpErrorViewModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
