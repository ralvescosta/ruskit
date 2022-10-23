use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum EventsError {
    #[error("collectors internal error")]
    InternalError,

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

    #[error("error to deserialization hourmeter message - `{0}`")]
    HourmeterMessageDeserializationError(String),

    #[error("error to deserialization routing created message - `{0}`")]
    RoutingCreatedMessageDeserializationError(String),

    #[error("error to deserialization routing created message - `{0}`")]
    RoutingUpdatedMessageDeserializationError(String),

    #[error("error to deserialization routing created message - `{0}`")]
    RoutingDeletedMessageDeserializationError(String),
}
