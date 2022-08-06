use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MqttError {
    #[error("mqtt internal error")]
    InternalError,

    #[error("mqtt unknown message kind")]
    UnknownMessageKindError,

    #[error("mqtt unformatted topic")]
    UnformattedTopicError,

    #[error("mqtt failure to publish in a topic")]
    PublishingError,

    #[error("mqtt failure to subscribe in a topic")]
    SubscribeError,
}
