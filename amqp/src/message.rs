use crate::errors::AmqpError;
use serde::Serialize;

pub trait AmqpMessage {
    fn get_type(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct AmqpPayload {
    pub payload: Box<[u8]>,
    pub msg_type: String,
}

impl AmqpPayload {
    pub fn new<T>(payload: &T) -> Result<Self, AmqpError>
    where
        T: AmqpMessage + Serialize,
    {
        let serialized = serde_json::to_vec::<T>(&payload)
            .map_err(|_| AmqpError::ParsePayloadError {})?
            .into_boxed_slice();

        Ok(AmqpPayload {
            msg_type: payload.get_type(),
            payload: serialized,
        })
    }
}
