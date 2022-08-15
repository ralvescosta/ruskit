use super::types::{Controller, Messages, QoS, Topic};
use async_trait::async_trait;
use env::Config;
use errors::mqtt::MqttError;
use log::{debug, error};
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::FutureExt,
    Context,
};
use otel;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[async_trait]
pub trait IMQTT {
    fn connect(&mut self, cfg: &Config) -> EventLoop;
    async fn subscriber(
        &mut self,
        topic: String,
        label: String,
        qos: QoS,
        controller: Arc<dyn Controller + Sync + Send>,
    ) -> Result<(), MqttError>;
    async fn publish(
        &self,
        ctx: &Context,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: &[u8],
    ) -> Result<(), MqttError>;
    async fn handle_event(&self, event: &Event) -> Result<(), MqttError>;
}

pub struct MQTT {
    client: Option<Box<AsyncClient>>,
    dispatchers: HashMap<String, Arc<dyn Controller + Sync + Send>>,
    tracer: BoxedTracer,
}

impl MQTT {
    pub fn new() -> Box<dyn IMQTT + Send + Sync> {
        Box::new(MQTT {
            client: None,
            dispatchers: HashMap::default(),
            tracer: global::tracer("mqtt"),
        })
    }

    #[cfg(test)]
    pub fn mock(
        dispatchers: HashMap<String, Arc<dyn Controller + Sync + Send>>,
    ) -> Box<dyn IMQTT + Send + Sync> {
        Box::new(MQTT {
            client: None,
            dispatchers,
            tracer: global::tracer("mqtt"),
        })
    }
}

#[async_trait]
impl IMQTT for MQTT {
    fn connect(&mut self, cfg: &Config) -> EventLoop {
        let mut mqtt_options = MqttOptions::new(cfg.app_name, cfg.mqtt_host, cfg.mqtt_port);

        mqtt_options
            .set_credentials(cfg.mqtt_user, cfg.mqtt_password)
            .set_keep_alive(Duration::from_secs(5));

        let (client, eventloop) = AsyncClient::new(mqtt_options, 50);

        self.client = Some(Box::new(client));

        eventloop
    }

    async fn subscriber(
        &mut self,
        topic: String,
        label: String,
        qos: QoS,
        controller: Arc<dyn Controller + Sync + Send>,
    ) -> Result<(), MqttError> {
        debug!("subscribing in topic: {:?}...", topic);

        self.client
            .clone()
            .unwrap()
            .subscribe(topic.clone(), qos.try_to())
            .await
            .map_err(|_| MqttError::SubscribeError {})?;

        self.dispatchers.insert(label, controller);

        debug!("subscribed");
        Ok(())
    }

    async fn publish(
        &self,
        ctx: &Context,
        topic: &str,
        qos: QoS,
        retain: bool,
        payload: &[u8],
    ) -> Result<(), MqttError> {
        debug!("publishing in a topic {:?}", topic);

        let cx = otel::tracing::ctx_from_ctx(&self.tracer, ctx, "mqtt publish");

        self.client
            .clone()
            .unwrap()
            .publish(topic, qos.try_to(), retain, payload)
            .with_context(cx)
            .await
            .map_err(|_| MqttError::PublishingError {})?;

        debug!("message published");
        Ok(())
    }

    async fn handle_event(&self, event: &Event) -> Result<(), MqttError> {
        if let Event::Incoming(Packet::Publish(msg)) = event.to_owned() {
            debug!("message received in a topic {:?}", msg.topic);

            let metadata =
                Topic::new(&msg.topic).map_err(|_| MqttError::UnformattedTopicError {})?;

            debug!("metadata: {:?}", metadata);

            let data = Messages::from_payload(&metadata, &msg.payload)?;

            debug!("msg: {:?}", data);

            let name = Box::new(format!("{:?}", msg.topic));
            let ctx = otel::tracing::new_ctx(&self.tracer, Box::leak(name));

            let controller = self.dispatchers.get(metadata.label);
            if controller.is_none() {
                error!("cant find controller for this topic");
                return Err(MqttError::InternalError {});
            }

            return match controller.unwrap().exec(&ctx, &data).await {
                Ok(_) => {
                    debug!("event processed successfully");
                    // span.set_status(StatusCode::Ok, format!("event processed successfully"));
                    Ok(())
                }
                Err(e) => {
                    error!("failed to handle the event - {:?}", e);
                    // span.set_status(StatusCode::Error, format!("failed to handle the event"));
                    Err(e)
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Controller;
    use async_trait::async_trait;
    use bytes::Bytes;
    use rumqttc::Publish;

    #[test]
    fn should_connect() {
        let mut mq = MQTT::new();
        mq.connect(&Config::mock());
    }

    #[tokio::test]
    async fn should_handle_event_successfully() {
        let mocked_controller = MockController::new(None);

        let mut map: HashMap<String, Arc<dyn Controller + Sync + Send>> = HashMap::default();

        map.insert("".to_owned(), Arc::new(mocked_controller));

        let mq = MQTT::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("{\"temp\": 40.2, \"time\": 10101010}").unwrap(),
            pkid: 10,
            qos: QoS::AtMostOnce.try_to(),
            retain: false,
            topic: "C2B/data/temp/device_id".to_owned(),
        }));

        let res = mq.handle_event(&event).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn should_return_err_if_topic_is_unformatted() {
        let mocked_controller = MockController::new(None);

        let mut map: HashMap<String, Arc<dyn Controller + Sync + Send>> = HashMap::default();

        map.insert("".to_owned(), Arc::new(mocked_controller));

        let mq = MQTT::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("{temp: 40.2, time: 10101010}").unwrap(),
            pkid: 10,
            qos: QoS::AtMostOnce.try_to(),
            retain: false,
            topic: "C2B/data/".to_owned(),
        }));

        let res = mq.handle_event(&event).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_return_err_if_message_is_unformatted() {
        let mocked_controller = MockController::new(None);

        let mut map: HashMap<String, Arc<dyn Controller + Sync + Send>> = HashMap::default();

        map.insert("".to_owned(), Arc::new(mocked_controller));

        let mq = MQTT::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("").unwrap(),
            pkid: 10,
            qos: QoS::AtMostOnce.try_to(),
            retain: false,
            topic: "C2B/data/collector/device".to_owned(),
        }));

        let res = mq.handle_event(&event).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_return_err_if_controller_return_err() {
        let mocked_controller = MockController::new(Some(MqttError::InternalError {}));

        let mut map: HashMap<String, Arc<dyn Controller + Sync + Send>> = HashMap::default();

        map.insert("".to_owned(), Arc::new(mocked_controller));

        let mq = MQTT::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("{temp: 40.2, time: 10101010}").unwrap(),
            pkid: 10,
            qos: QoS::AtMostOnce.try_to(),
            retain: false,
            topic: "C2B/data/collector/device".to_owned(),
        }));

        let res = mq.handle_event(&event).await;
        assert!(res.is_err());
    }

    struct MockController {
        pub mock_error: Option<MqttError>,
    }

    impl MockController {
        pub fn new(err: Option<MqttError>) -> MockController {
            MockController { mock_error: err }
        }
    }

    #[async_trait]
    impl Controller for MockController {
        async fn exec(&self, _ctx: &Context, _msg: &Messages) -> Result<(), MqttError> {
            if self.mock_error.is_none() {
                return Ok(());
            }

            return Err(MqttError::InternalError {});
        }
    }
}
