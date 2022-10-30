use errors::mqtt::MqttError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct TopicMessage<'t> {
    pub topic: &'t str,
    pub label: &'t str,
    pub device: &'t str,
    pub rest: &'t str,
}

impl<'t> TopicMessage<'t> {
    pub fn new(topic: &'t str) -> Result<TopicMessage<'t>, MqttError> {
        let splitted = topic.split("/").collect::<Vec<&str>>();
        if splitted.len() <= 3 {
            return Err(MqttError::UnformattedTopicError {});
        }

        Ok(TopicMessage {
            topic,
            label: splitted[0],
            device: splitted[1],
            rest: splitted[2],
        })
    }
}
