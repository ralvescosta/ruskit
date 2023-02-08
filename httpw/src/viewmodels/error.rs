use serde::Serialize;

#[derive(Default, Serialize)]
pub struct HttpErrorViewModel {
    pub status_code: u16,
    pub message: String,
    pub details: String,
}
