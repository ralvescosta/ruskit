use async_trait::async_trait;
use bytes::Bytes;
use errors::mqtt::MqttError;
use events::mqtt::TopicMessage;
use opentelemetry::Context;

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

#[async_trait]
pub trait Controller {
    async fn exec(
        &self,
        ctx: &Context,
        msgs: &Bytes,
        topic: &TopicMessage,
    ) -> Result<(), MqttError>;
}

#[async_trait]
pub trait ControllerV2 {
    async fn exec(&self, ctx: &Context, msgs: &[u8], topic: &TopicMessage)
        -> Result<(), MqttError>;
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
