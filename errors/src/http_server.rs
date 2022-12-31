use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HttpServerError {
    #[error("http server error")]
    ServerError,

    #[error("http port bind error")]
    HttpPortBindError,
}
