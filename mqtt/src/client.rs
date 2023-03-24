use crate::{errors::MQTTError, types::MqttPayload};
use async_trait::async_trait;
#[cfg(test)]
use mockall::*;
#[cfg(mock)]
use mockall::*;
use paho_mqtt::{AsyncClient, AsyncReceiver, Message, MessageBuilder};
use std::sync::Arc;
use tracing::{debug, error};

#[cfg_attr(test, automock)]
#[cfg_attr(mock, automock)]
#[async_trait]
pub trait MQTTClient {
    async fn publish(&self, topic: &str, qos: i32, payload: &MqttPayload) -> Result<(), MQTTError>;
    async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), MQTTError>;
    fn get_stream(&mut self) -> AsyncReceiver<Option<Message>>;
    fn get_client(&self) -> Arc<AsyncClient>;
}

pub struct MQTTClientImpl {
    pub client: Arc<AsyncClient>,
    pub(super) stream: AsyncReceiver<Option<Message>>,
}

#[async_trait]
impl MQTTClient for MQTTClientImpl {
    async fn publish(&self, topic: &str, qos: i32, payload: &MqttPayload) -> Result<(), MQTTError> {
        if !self.client.is_connected() {
            error!("connection to mqtt broker was lost");
            return Err(MQTTError::ConnectionLostError {});
        }

        debug!("publishing to topic {}", topic);

        self.client.publish(
            MessageBuilder::new()
                .topic(topic)
                .qos(qos)
                .payload(payload.0.clone())
                .finalize(),
        );

        debug!("message published");

        Ok(())
    }

    async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), MQTTError> {
        if !self.client.is_connected() {
            error!("connection to mqtt broker was lost");
            return Err(MQTTError::ConnectionLostError {});
        }

        debug!("subscribing to the topic {}", topic);

        self.client.subscribe(topic, qos);

        debug!("subscribed");

        Ok(())
    }

    fn get_stream(&mut self) -> AsyncReceiver<Option<Message>> {
        self.stream.clone()
    }

    fn get_client(&self) -> Arc<AsyncClient> {
        self.client.clone()
    }
}
