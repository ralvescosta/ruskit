use crate::{
    client::MQTTClient,
    errors::MQTTError,
    types::{Controller, TopicMessage},
};
use futures_util::StreamExt;
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{SpanKind, Status, TraceContextExt},
    Context,
};
use paho_mqtt::Message;
use std::{borrow::Cow, sync::Arc};
use tracing::{debug, error, warn};

pub struct MqttDispatcher {
    pub(crate) topics: Vec<String>,
    pub(crate) dispatches: Vec<Arc<dyn Controller + Sync + Send>>,
    pub(crate) tracer: BoxedTracer,
}

impl MqttDispatcher {
    pub fn new() -> Self {
        MqttDispatcher {
            topics: vec![],
            dispatches: vec![],
            tracer: global::tracer("mqtt_consumer"),
        }
    }

    pub fn declare(
        &mut self,
        topic: &str,
        dispatch: Arc<dyn Controller + Send + Sync>,
    ) -> Result<(), MQTTError> {
        if topic.is_empty() {
            return Err(MQTTError::DispatcherError {});
        }

        self.topics.push(topic.to_owned());
        self.dispatches.push(dispatch);

        Ok(())
    }

    async fn consume(&self, ctx: &Context, msg: &Message) -> Result<(), MQTTError> {
        let dispatch_index = self.get_dispatch_index(ctx, msg.topic())?;

        let metadata = TopicMessage::new(msg.topic())?;

        let ctx = traces::span_ctx(&self.tracer, SpanKind::Consumer, msg.topic());
        let span = ctx.span();

        debug!(
            trace.id = traces::trace_id(&ctx),
            span.id = traces::span_id(&ctx),
            "message received in a topic {:?}",
            msg.topic()
        );

        let dispatch = self.dispatches.get(dispatch_index).unwrap();

        return match dispatch.exec(&ctx, msg.payload(), &metadata).await {
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
                span.set_status(Status::Error {
                    description: Cow::from("failed to handle the event"),
                });
                Err(e)
            }
        };
    }

    pub async fn consume_blocking(&self, mut client: Box<dyn MQTTClient>) -> Result<(), MQTTError> {
        for topic in self.topics.clone() {
            client.subscribe(&topic, 1).await?;
        }

        let mut stream = client.get_stream();

        while let Some(delivery) = stream.next().await {
            match delivery {
                Some(msg) => match self.consume(&Context::new(), &msg).await {
                    Err(e) => error!(error = e.to_string(), ""),
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(())
    }
}

impl MqttDispatcher {
    fn get_dispatch_index(&self, ctx: &Context, received_topic: &str) -> Result<usize, MQTTError> {
        let mut p: i16 = -1;
        for handler_topic_index in 0..self.topics.len() {
            let handler_topic = self.topics[handler_topic_index].clone();

            if received_topic == handler_topic {
                p = handler_topic_index as i16;
                break;
            }

            if received_topic.len() > received_topic.len() {
                break;
            }

            let handler_fields: Vec<_> = handler_topic.split('/').collect();
            let received_fields: Vec<_> = received_topic.split('/').collect();

            for i in 0..handler_fields.len() {
                if handler_fields[i] == "#" {
                    p = handler_topic_index as i16;
                    break;
                }

                if handler_fields[i] != "+" && handler_fields[i] != received_fields[i] {
                    break;
                }

                if handler_fields[i] == "+" && i == handler_fields.len() - 1 {
                    p = handler_topic_index as i16;
                }
            }

            if handler_fields.len() == received_fields.len() {
                p = handler_topic_index as i16;
                break;
            }
        }

        if p == -1 {
            warn!(
                trace.id = traces::trace_id(&ctx),
                span.id = traces::span_id(&ctx),
                "cant find dispatch for this topic"
            );
            return Err(MQTTError::UnregisteredDispatchForThisTopicError(
                received_topic.to_owned(),
            ));
        }

        Ok(p as usize)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::errors::MQTTError;
    use async_trait::async_trait;

    #[test]
    fn test_new() {
        MqttDispatcher::new();
    }

    #[test]
    fn test_declare() {
        let mut dispatch = MqttDispatcher::new();

        let res = dispatch.declare("some/topic", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let res = dispatch.declare("", Arc::new(MockDispatch::new()));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_consume() {
        let mut dispatch = MqttDispatcher::new();

        let res = dispatch.declare("some/topic/#", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let msg = Message::new("some/topic/sub/1", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_consume_with_plus_wildcard() {
        let mut dispatch = MqttDispatcher::new();

        let res = dispatch.declare("some/+/+/sub", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let msg = Message::new("some/topic/with/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_consume_with_dispatch_return_err() {
        let mut dispatch = MqttDispatcher::new();

        let mut mock = MockDispatch::new();
        mock.set_error(MQTTError::InternalError {});

        let res = dispatch.declare("/some/topic/#", Arc::new(mock));
        assert!(res.is_ok());

        let msg = Message::new("/some/topic/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_consume_with_unregistered_consumer() {
        let mut dispatch = MqttDispatcher::new();

        let res = dispatch.declare("other/topic/#", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let msg = Message::new("some/topic/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_err());
    }

    struct MockDispatch {
        error: Option<MQTTError>,
    }

    impl MockDispatch {
        pub fn new() -> Self {
            MockDispatch { error: None }
        }

        pub fn set_error(&mut self, err: MQTTError) {
            self.error = Some(err)
        }
    }

    #[async_trait]
    impl Controller for MockDispatch {
        async fn exec(
            &self,
            _ctx: &Context,
            _msgs: &[u8],
            _topic: &TopicMessage,
        ) -> Result<(), MQTTError> {
            if self.error.is_some() {
                return Err(self.error.clone().unwrap());
            }

            Ok(())
        }
    }
}
