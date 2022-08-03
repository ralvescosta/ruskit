use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AmqpError {
    #[error("failure to connect")]
    ConnectionError,

    #[error("failure to create a channel")]
    ChannelError,

    #[error("failure to declare an exchange `{0}`")]
    DeclareExchangeError(String),

    #[error("failure to declare a queue `{0}`")]
    DeclareQueueError(String),

    #[error("failure to binding exchange `{0}` to queue `{0}`")]
    BindingExchangeToQueueError(String, String),

    #[error("failure to declare consumer `{0}`")]
    BindingConsumerError(String),

    #[error("failure to publish")]
    PublishingError,

    #[error("failure to parse payload")]
    ParsePayloadError,

    #[error("failure to ack message")]
    AckMessageError,

    #[error("failure to nack message")]
    NackMessageError,

    #[error("failure to requeuing message")]
    RequeuingMessageError,

    #[error("failure to publish to dlq")]
    PublishingToDQLError,
}
