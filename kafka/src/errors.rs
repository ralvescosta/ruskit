use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum KafkaError {
    #[error("internal error")]
    InternalError,
}
