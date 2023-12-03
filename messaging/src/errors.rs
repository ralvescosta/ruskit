use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MessagingError {
    #[error("internal error")]
    InternalError,

    #[error("failure to connect")]
    ConnectionError,

    #[error("failure to create the consumer")]
    CreatingConsumerError,

    #[error("failure to consume message `{0}`")]
    ConsumerError(String),
}
