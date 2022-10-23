use errors::events::EventsError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct TopicMessage<'t> {
    pub topic: &'t str,
    pub label: &'t str,
    pub organization_id: &'t str,
    pub network: &'t str,
    pub collector_id: &'t str,
    pub is_ack: bool,
}

impl<'t> TopicMessage<'t> {
    pub fn new(topic: &'t str) -> Result<TopicMessage<'t>, EventsError> {
        let splitted = topic.split("/").collect::<Vec<&str>>();
        if splitted.len() <= 3 {
            return Err(EventsError::InternalError {});
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
            topic,
            label,
            organization_id,
            network,
            collector_id,
            is_ack,
        })
    }
}
