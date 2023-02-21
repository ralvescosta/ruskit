use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LoggingError {
    #[error("logging internal error")]
    InternalError,
}
