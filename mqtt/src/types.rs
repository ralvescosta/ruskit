use async_trait::async_trait;
use errors::mqtt::MqttError;
use opentelemetry::Context;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Clone, Default)]
pub enum BrokerKind {
    #[default]
    SelfHostedWithPassword,
    SelfHostedWithoutPassword,
    AWSIoTCore,
}

pub struct MqttPayload(pub Box<[u8]>);

impl MqttPayload {
    pub fn new<T>(data: &T) -> Result<MqttPayload, MqttError>
    where
        T: serde::Serialize,
    {
        let bytes = serde_json::to_vec(data).map_err(|e| {
            error!(error = e.to_string(), "error parsing payload");
            MqttError::SerializePayloadError(e.to_string())
        })?;

        Ok(MqttPayload(bytes.into_boxed_slice()))
    }
}

#[async_trait]
pub trait Controller {
    async fn exec(&self, ctx: &Context, msgs: &[u8], topic: &TopicMessage)
        -> Result<(), MqttError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct TopicMessage {
    pub topic: String,
    pub label: String,
    pub organization_id: String,
    pub network: String,
    pub collector_id: String,
    pub is_ack: bool,
}

impl TopicMessage {
    pub fn new(topic: &str) -> Result<TopicMessage, MqttError> {
        let splitted = topic.split("/").collect::<Vec<&str>>();
        if splitted.len() <= 3 {
            return Err(MqttError::UnformattedTopicError {});
        }

        let mut is_ack = false;
        if splitted.len() == 5 {
            is_ack = true;
        }

        let label = splitted[0];
        let organization_id = splitted[1];
        let network = splitted[2];
        let collector_id = splitted[3];

        Ok(TopicMessage {
            topic: topic.to_owned(),
            label: label.to_owned(),
            organization_id: organization_id.to_owned(),
            network: network.to_owned(),
            collector_id: collector_id.to_owned(),
            is_ack,
        })
    }
}

#[cfg(test)]
mod tests {}
