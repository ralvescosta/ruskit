use async_trait::async_trait;
use bytes::Bytes;
use errors::mqtt::MqttError;
use opentelemetry::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl QoS {
    pub fn to(qos: QoS) -> rumqttc::QoS {
        match qos {
            QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
            QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
            QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
        }
    }

    pub fn try_to(&self) -> rumqttc::QoS {
        match self {
            QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
            QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
            QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
        }
    }

    pub fn from(qos: rumqttc::QoS) -> Self {
        match qos {
            rumqttc::QoS::AtMostOnce => QoS::AtMostOnce,
            rumqttc::QoS::AtLeastOnce => QoS::AtLeastOnce,
            rumqttc::QoS::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Topic<'t> {
    pub topic: &'t str,
    pub label: &'t str,
    //@TODO: create an enum for message kind
    pub kind: &'t str,
}

impl<'t> Topic<'t> {
    pub fn new(topic: &str) -> Result<Topic, MqttError> {
        return Ok(Topic {
            topic,
            label: "",
            kind: "temp",
        });
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Messages {
    Temp(TempMessage),
}

impl Messages {
    pub fn from_payload(topic: &Topic, payload: &Bytes) -> Result<Messages, MqttError> {
        match topic.kind {
            "temp" => {
                let t = serde_json::from_slice::<TempMessage>(payload);

                Ok(Messages::Temp(t.map_err(|_| MqttError::InternalError {})?))
            }
            _ => Err(MqttError::UnknownMessageKindError {}),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TempMessage {
    pub temp: f32,
    pub time: u64,
}

#[async_trait]
pub trait Controller {
    async fn exec(&self, ctx: &Context, msg: &Messages) -> Result<(), MqttError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_qos_correctly() {
        assert_eq!(QoS::from(rumqttc::QoS::AtLeastOnce), QoS::AtLeastOnce);
        assert_eq!(QoS::from(rumqttc::QoS::AtMostOnce), QoS::AtMostOnce);
        assert_eq!(QoS::from(rumqttc::QoS::ExactlyOnce), QoS::ExactlyOnce);

        assert_eq!(QoS::to(QoS::AtLeastOnce), rumqttc::QoS::AtLeastOnce);
        assert_eq!(QoS::to(QoS::AtMostOnce), rumqttc::QoS::AtMostOnce);
        assert_eq!(QoS::to(QoS::ExactlyOnce), rumqttc::QoS::ExactlyOnce);

        assert_eq!(QoS::AtLeastOnce.try_to(), rumqttc::QoS::AtLeastOnce);
        assert_eq!(QoS::AtMostOnce.try_to(), rumqttc::QoS::AtMostOnce);
        assert_eq!(QoS::ExactlyOnce.try_to(), rumqttc::QoS::ExactlyOnce);
    }
}
