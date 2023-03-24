use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum MQTTError {
    #[error("mqtt internal error")]
    InternalError,

    #[error("mqtt connection error")]
    ConnectionError,

    #[error("mqtt broker connection lost")]
    ConnectionLostError,

    #[error("mqtt unregistered dispatch for this topic: `{0}`")]
    UnregisteredDispatchForThisTopicError(String),

    #[error("mqtt serialize payload error: `{0}`")]
    SerializePayloadError(String),

    #[error("mqtt failure to deserialize message - MqttError::DeserializeMessageError: `{0}`")]
    DeserializeMessageError(String),

    #[error("mqtt unformatted topic")]
    UnformattedTopicError,

    #[error("controller for this topic was not founded")]
    TopicControllerWasNotFound,

    #[error("mqtt failure to publish in a topic")]
    PublishingError,

    #[error("mqtt failure to subscribe in a topic")]
    SubscribeError,

    #[error("mqtt dispatcher error")]
    DispatcherError,

    #[error("error to deserialization ack message - AcksMessageDeserializationError: `{0}`")]
    AckMessageDeserializationError(String),

    #[error("error to deserialization collector log message - CollectorLogMessageDeserializationError: `{0}`")]
    CollectorLogMessageDeserializationError(String),

    #[error("error to deserialization iot base message - IoTBaseMessageDeserializationError: `{0}` msg: {1}")]
    IoTBaseMessageDeserializationError(String, String),

    #[error("error to deserialization iot prov available message - IoTProvAvailableMessageDeserializationError: `{0}`")]
    IoTProvAvailableMessageDeserializationError(String),

    #[error(
        "error to deserialization iot single message - IoTSingleMessageDeserializationError: `{0}`"
    )]
    IoTSingleMessageDeserializationError(String),

    #[error("error to deserialization iot multiple message - IoTMultipleMessageDeserializationError: `{0}`")]
    IoTMultipleMessageDeserializationError(String),
}
