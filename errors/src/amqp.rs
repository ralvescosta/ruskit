use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AmqpError {
    #[error("internal error")]
    InternalError,

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

    #[error("failure to configure qos `{0}`")]
    QoSDeclarationError(String),

    #[error("consumer declaration error")]
    ConsumerDeclarationError,

    #[error("failure to consume message `{0}`")]
    ConsumerError(String),

    #[error("error to deserialization iot prov available message - IoTProvAvailableMessageDeserializationError: `{0}`")]
    IoTProvAvailableMessageDeserializationError(String),

    #[error("error to deserialization ack message - AcksMessageDeserializationError: `{0}`")]
    AckMessageDeserializationError(String),

    #[error(
        "error to deserialization iot single message - IoTSingleMessageDeserializationError: `{0}`"
    )]
    IoTSingleMessageDeserializationError(String),

    #[error("error to deserialization iot multiple message - IoTMultipleMessageDeserializationError: `{0}`")]
    IoTMultipleMessageDeserializationError(String),

    #[error("error to deserialization iot multiple message timer - IoTMultipleMessageTimerDeserializationError: `{0}`")]
    IoTMultipleMessageTimerDeserializationError(String),
}
