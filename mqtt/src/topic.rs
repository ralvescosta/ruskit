use crate::errors::MQTTError;
use serde::{Deserialize, Serialize};

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
    pub fn new(topic: &str) -> Result<TopicMessage, MQTTError> {
        let splitted = topic.split("/").collect::<Vec<&str>>();
        if splitted.len() <= 3 {
            return Err(MQTTError::UnformattedTopicError {});
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
