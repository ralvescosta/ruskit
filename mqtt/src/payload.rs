use crate::errors::MQTTError;
use tracing::error;

pub struct Payload(pub Box<[u8]>);

impl Payload {
    pub fn new<T>(data: &T) -> Result<Payload, MQTTError>
    where
        T: serde::Serialize,
    {
        let bytes = serde_json::to_vec(data).map_err(|e| {
            error!(error = e.to_string(), "error parsing payload");
            MQTTError::SerializePayloadError(e.to_string())
        })?;

        Ok(Payload(bytes.into_boxed_slice()))
    }
}
