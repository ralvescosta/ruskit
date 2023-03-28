use crate::errors::MQTTError;
use async_trait::async_trait;
use futures_util::StreamExt;
#[cfg(test)]
use mockall::*;
#[cfg(feature = "mocks")]
use mockall::*;
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{SpanKind, Status, TraceContextExt},
    Context,
};
use paho_mqtt::{AsyncClient, AsyncReceiver, Message};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, sync::Arc};
use tracing::{debug, error, warn};

#[cfg_attr(test, automock)]
#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait ConsumerHandler {
    async fn exec(&self, ctx: &Context, msgs: &[u8], topic: &TopicMessage)
        -> Result<(), MQTTError>;
}

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

pub struct MQTTDispatcher {
    conn: Arc<AsyncClient>,
    stream: AsyncReceiver<Option<Message>>,
    pub(crate) topics: Vec<String>,
    pub(crate) dispatches: Vec<Arc<dyn ConsumerHandler + Sync + Send>>,
    pub(crate) tracer: BoxedTracer,
}

impl MQTTDispatcher {
    pub fn new(conn: Arc<AsyncClient>, stream: AsyncReceiver<Option<Message>>) -> Self {
        MQTTDispatcher {
            conn,
            stream,
            topics: vec![],
            dispatches: vec![],
            tracer: global::tracer("mqtt_consumer"),
        }
    }

    pub fn declare(
        &mut self,
        topic: &str,
        dispatch: Arc<dyn ConsumerHandler + Send + Sync>,
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

    pub async fn consume_blocking(&mut self) -> Result<(), MQTTError> {
        for topic in self.topics.clone() {
            self.conn.subscribe(topic, 1);
        }

        while let Some(delivery) = self.stream.next().await {
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

impl MQTTDispatcher {
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
    use paho_mqtt::CreateOptions;

    #[test]
    fn test_new() {
        let mut client = AsyncClient::new(CreateOptions::default()).unwrap();
        let stream = client.get_stream(2048);
        MQTTDispatcher::new(Arc::new(client), stream);
    }

    #[test]
    fn test_declare() {
        let mut client = AsyncClient::new(CreateOptions::default()).unwrap();
        let stream = client.get_stream(2048);
        let mut dispatch = MQTTDispatcher::new(Arc::new(client), stream);

        let res = dispatch.declare("some/topic", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let res = dispatch.declare("", Arc::new(MockDispatch::new()));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_consume() {
        let mut client = AsyncClient::new(CreateOptions::default()).unwrap();
        let stream = client.get_stream(2048);
        let mut dispatch = MQTTDispatcher::new(Arc::new(client), stream);

        let res = dispatch.declare("some/topic/#", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let msg = Message::new("some/topic/sub/1", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_consume_with_plus_wildcard() {
        let mut client = AsyncClient::new(CreateOptions::default()).unwrap();
        let stream = client.get_stream(2048);
        let mut dispatch = MQTTDispatcher::new(Arc::new(client), stream);

        let res = dispatch.declare("some/+/+/sub", Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let msg = Message::new("some/topic/with/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_consume_with_dispatch_return_err() {
        let mut client = AsyncClient::new(CreateOptions::default()).unwrap();
        let stream = client.get_stream(2048);
        let mut dispatch = MQTTDispatcher::new(Arc::new(client), stream);

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
        let mut client = AsyncClient::new(CreateOptions::default()).unwrap();
        let stream = client.get_stream(2048);
        let mut dispatch = MQTTDispatcher::new(Arc::new(client), stream);

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
    impl ConsumerHandler for MockDispatch {
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
