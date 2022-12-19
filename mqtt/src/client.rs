use crate::types::MqttPayload;
use async_trait::async_trait;
use errors::mqtt::MqttError;
use paho_mqtt::{AsyncClient, AsyncReceiver, Message, MessageBuilder};
use std::sync::Arc;
use tracing::{debug, error};

#[async_trait]
pub trait MqttClient {
    async fn publish(&self, topic: &str, qos: i32, payload: &MqttPayload) -> Result<(), MqttError>;
    async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), MqttError>;
    fn get_stream(&mut self) -> AsyncReceiver<Option<Message>>;
    fn get_client(&self) -> Arc<AsyncClient>;
}

pub struct MqttClientImpl {
    pub client: Arc<AsyncClient>,
    pub(super) stream: AsyncReceiver<Option<Message>>,
}

#[async_trait]
impl MqttClient for MqttClientImpl {
    async fn publish(&self, topic: &str, qos: i32, payload: &MqttPayload) -> Result<(), MqttError> {
        if !self.client.is_connected() {
            error!("connection to mqtt broker was lost");
            return Err(MqttError::ConnectionLostError {});
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

    async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), MqttError> {
        if !self.client.is_connected() {
            error!("connection to mqtt broker was lost");
            return Err(MqttError::ConnectionLostError {});
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
