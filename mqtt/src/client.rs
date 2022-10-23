use super::types::{Controller, QoS};
use async_trait::async_trait;
use env::Config;
use errors::mqtt::MqttError;
use events::mqtt::TopicMessage;
#[cfg(test)]
use mockall::predicate::*;
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{FutureExt, SpanKind, TraceContextExt},
    Context,
};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet};
use std::{collections::HashMap, sync::Arc, time::Duration};
use traces;
use tracing::{debug, error};

#[async_trait]
pub trait MQTT {
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

pub struct MQTTImpl {
    client: Option<Box<AsyncClient>>,
    dispatchers: HashMap<String, Arc<dyn Controller + Sync + Send>>,
    tracer: BoxedTracer,
}

impl MQTTImpl {
    pub fn new() -> Box<dyn MQTT + Send + Sync> {
        Box::new(MQTTImpl {
            client: None,
            dispatchers: HashMap::default(),
            tracer: global::tracer("mqtt"),
        })
    }

    #[cfg(test)]
    pub fn mock(
        dispatchers: HashMap<String, Arc<dyn Controller + Sync + Send>>,
    ) -> Box<dyn MQTT + Send + Sync> {
        Box::new(MQTTImpl {
            client: None,
            dispatchers,
            tracer: global::tracer("mqtt"),
        })
    }
}

#[async_trait]
impl MQTT for MQTTImpl {
    fn connect(&mut self, cfg: &Config) -> EventLoop {
        debug!("connection to mqtt broker...");
        let mut mqtt_options = MqttOptions::new(&cfg.app_name, &cfg.mqtt_host, cfg.mqtt_port);

        mqtt_options
            .set_credentials(&cfg.mqtt_user, &cfg.mqtt_password)
            .set_keep_alive(Duration::from_secs(5));

        let (client, eventloop) = AsyncClient::new(mqtt_options, 50);

        self.client = Some(Box::new(client));

        debug!("connected to mqtt broker");

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

        self.client
            .clone()
            .unwrap()
            .publish(topic, qos.try_to(), retain, payload)
            .with_context(ctx.to_owned())
            .await
            .map_err(|_| MqttError::PublishingError {})?;

        debug!("message published");
        Ok(())
    }

    async fn handle_event(&self, event: &Event) -> Result<(), MqttError> {
        if let Event::Incoming(Packet::Publish(msg)) = event.to_owned() {
            let metadata =
                TopicMessage::new(&msg.topic).map_err(|_| MqttError::UnformattedTopicError {})?;

            let name = format!("{:?}", msg.topic);
            let ctx = traces::span_ctx(&self.tracer, SpanKind::Consumer, &name);
            let span = ctx.span();

            debug!(
                trace.id = traces::trace_id(&ctx),
                span.id = traces::span_id(&ctx),
                "message received in a topic {:?}",
                msg.topic
            );

            let controller = self.dispatchers.get(metadata.label);
            if controller.is_none() {
                error!(
                    trace.id = traces::trace_id(&ctx),
                    span.id = traces::span_id(&ctx),
                    "cant find controller for this topic"
                );
                return Err(MqttError::TopicControllerWasNotFound {});
            }

            return match controller
                .unwrap()
                .exec(&ctx, &msg.payload, &metadata)
                .await
            {
                Ok(_) => {
                    debug!(
                        trace.id = traces::trace_id(&ctx),
                        span.id = traces::span_id(&ctx),
                        "event processed successfully"
                    );
                    Ok(())
                }
                Err(e) => {
                    debug!(
                        trace.id = traces::trace_id(&ctx),
                        span.id = traces::span_id(&ctx),
                        "failed to handle the event - {:?}",
                        e
                    );
                    span.record_error(&e);
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
        let mut mq = MQTTImpl::new();
        mq.connect(&Config::mock());
    }

    #[tokio::test]
    async fn should_handle_event_successfully() {
        let mocked_controller = MockController::new(None);

        let mut map: HashMap<String, Arc<dyn Controller + Sync + Send>> = HashMap::default();

        map.insert("C2B".to_owned(), Arc::new(mocked_controller));

        let mq = MQTTImpl::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("[{\"mac\":\"454806000009\",\"time\":\"1660235235\",\"rssi\":\"-95\",\"adv\":\"036F03CAA201000000000000\"}]").unwrap(),
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

        map.insert("C2B".to_owned(), Arc::new(mocked_controller));

        let mq = MQTTImpl::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("[{\"mac\":\"454806000009\",\"time\":\"1660235235\",\"rssi\":\"-95\",\"adv\":\"036F03CAA201000000000000\"}]").unwrap(),
            pkid: 10,
            qos: QoS::AtMostOnce.try_to(),
            retain: false,
            topic: "C2B/data/".to_owned(),
        }));

        let res = mq.handle_event(&event).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_return_err_if_controller_return_err() {
        let mocked_controller = MockController::new(Some(MqttError::InternalError {}));

        let mut map: HashMap<String, Arc<dyn Controller + Sync + Send>> = HashMap::default();

        map.insert("C2B".to_owned(), Arc::new(mocked_controller));

        let mq = MQTTImpl::mock(map);

        let event = Event::Incoming(Packet::Publish(Publish {
            dup: true,
            payload: Bytes::try_from("[{\"mac\":\"454806000009\",\"time\":\"1660235235\",\"rssi\":\"-95\",\"adv\":\"036F03CAA201000000000000\"}]").unwrap(),
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
        async fn exec(
            &self,
            _ctx: &Context,
            _msg: &Bytes,
            _topic: &TopicMessage,
        ) -> Result<(), MqttError> {
            if self.mock_error.is_none() {
                return Ok(());
            }

            return Err(MqttError::InternalError {});
        }
    }
}
