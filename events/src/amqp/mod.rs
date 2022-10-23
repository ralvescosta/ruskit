use errors::amqp::AmqpError;
use serde::Serialize;

pub trait AmqpEvent {
    fn get_type(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct AmqpPublishData {
    pub payload: Box<[u8]>,
    pub msg_type: String,
}

impl AmqpPublishData {
    pub fn new<T>(payload: &T) -> Result<Self, AmqpError>
    where
        T: AmqpEvent + Serialize,
    {
        let serialized = serde_json::to_vec::<T>(&payload)
            .map_err(|_| AmqpError::ParsePayloadError {})?
            .into_boxed_slice();

        Ok(AmqpPublishData {
            msg_type: payload.get_type(),
            payload: serialized,
        })
    }
}
