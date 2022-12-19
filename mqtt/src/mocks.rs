use std::sync::Arc;

use crate::{client::MqttClient, types::MqttPayload};
use async_trait::async_trait;
use errors::mqtt::MqttError;
use mockall::*;
use paho_mqtt::{AsyncClient, AsyncReceiver, Message};

mock! {
  pub MqttClientImpl{}

  #[async_trait]
  impl MqttClient for MqttClientImpl {
    async fn publish(&self, topic: &str, qos: i32, payload: &MqttPayload) -> Result<(), MqttError> {
      todo!()
    }

    async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), MqttError> {
      todo!()
    }

    fn get_stream(&mut self) -> AsyncReceiver<Option<Message>> {
      todo!()
    }

    fn get_client(&self) -> Arc<AsyncClient>  {
      todo!()
    }
  }
}
