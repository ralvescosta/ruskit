use env::Config;
use errors::{amqp::AmqpError, mqtt::MqttError};
// use futures_util::StreamExt;
use mqtt::{AsyncClient, AsyncReceiver, Message, MessageBuilder};
use paho_mqtt as mqtt;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, error};

pub struct MqttImplV2 {
    client: AsyncClient,
    pub(super) stream: AsyncReceiver<Option<Message>>,
}

impl MqttImplV2 {
    pub async fn new(cfg: &Config) -> Result<Self, AmqpError> {
        let opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(&cfg.mqtt_host)
            .client_id(&cfg.app_name)
            .finalize();

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(false)
            .user_name(&cfg.mqtt_user)
            .password(&cfg.mqtt_password)
            .finalize();

        let mut client = mqtt::AsyncClient::new(opts).map_err(|e| {
            error!(error = e.to_string(), "error to create mqtt client");
            AmqpError::ConnectionError {}
        })?;

        let stream = client.get_stream(2048);

        client.connect(conn_opts.clone()).await.map_err(|e| {
            error!(error = e.to_string(), "error to create mqtt client");
            AmqpError::ConnectionError {}
        })?;

        Ok(MqttImplV2 { client, stream })
    }

    pub async fn publish<T>(&self, topic: &str, qos: i32, payload: &T) -> Result<(), MqttError>
    where
        T: Serialize,
    {
        let bytes = serde_json::to_vec(payload).map_err(|e| {
            error!(error = e.to_string(), "error parsing payload");
            MqttError::SerializePayloadError(e.to_string())
        })?;

        if !self.client.is_connected() {
            error!("connection to mqtt broker was lost");
            return Err(MqttError::ConnectionLostError {});
        }

        debug!("publishing to topic {}", topic);

        self.client.publish(
            MessageBuilder::new()
                .topic(topic)
                .qos(qos)
                .payload(bytes)
                .finalize(),
        );

        debug!("message published");

        Ok(())
    }

    pub async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), MqttError> {
        if !self.client.is_connected() {
            error!("connection to mqtt broker was lost");
            return Err(MqttError::ConnectionLostError {});
        }

        debug!("subscribing to the topic {}", topic);

        self.client.subscribe(topic, qos);

        debug!("subscribed");

        Ok(())
    }
}
