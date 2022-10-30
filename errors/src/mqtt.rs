use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum MqttError {
    #[error("mqtt internal error")]
    InternalError,

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
}
