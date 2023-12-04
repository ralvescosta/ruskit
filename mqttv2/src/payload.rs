use crate::errors::MQTTError;
use tracing::error;

pub struct Payload(pub Box<[u8]>);

impl Payload {
    pub fn new<T>(data: &T) -> Result<Payload, MQTTError>
    where
        T: serde::Serialize,
    {
        match serde_json::to_vec(data) {
            Err(err) => {
                error!(error = err.to_string(), "error parsing payload");
                Err(MQTTError::SerializePayloadError(err.to_string()))
            }
            Ok(bytes) => Ok(Payload(bytes.into_boxed_slice())),
        }
    }
}
