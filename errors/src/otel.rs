use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum OtelError {
    #[error("mqtt internal error")]
    InternalError,

    #[error("failed to read proc file")]
    ProcFileError,

    #[error("failed to register metric callback")]
    MetricCallbackError,
}
