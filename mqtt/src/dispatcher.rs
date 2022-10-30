use std::sync::Arc;

use errors::mqtt::MqttError;
use events::mqtt::TopicMessage;
use futures_util::StreamExt;
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{SpanKind, TraceContextExt},
    Context,
};
use paho_mqtt::{Message, TopicFilter};
use tracing::{debug, error, warn};

use crate::{client_v2::MqttImplV2, types::ControllerV2};

pub struct MqttDispatcher {
    pub(crate) topics: Vec<String>,
    pub(crate) dispatches: Vec<Arc<dyn ControllerV2 + Sync + Send>>,
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
        topic: String,
        dispatch: Arc<dyn ControllerV2 + Send + Sync>,
    ) -> Result<(), MqttError> {
        if topic.is_empty() {
            return Err(MqttError::DispatcherError {});
        }

        self.topics.push(topic);
        self.dispatches.push(dispatch);

        Ok(())
    }

    pub async fn consume(&self, ctx: &Context, msg: &Message) -> Result<(), MqttError> {
        let mut p = -1;
        for (i, tp) in self.topics.clone().into_iter().enumerate() {
            let filter = TopicFilter::new(tp).map_err(|e| {
                error!(
                    error = e.to_string(),
                    trace.id = traces::trace_id(&ctx),
                    span.id = traces::span_id(&ctx),
                    "error to create mqtt topic filter",
                );
                MqttError::InternalError {}
            })?;

            if filter.is_match(msg.topic()) {
                p = i as i8;
                break;
            }
        }

        if p == -1 {
            warn!(
                trace.id = traces::trace_id(&ctx),
                span.id = traces::span_id(&ctx),
                "cant find dispatch for this topic"
            );

            return Err(MqttError::UnregisteredDispatchForThisTopicError(
                msg.topic().to_owned(),
            ));
        }

        let metadata = TopicMessage::new(msg.topic())?;

        let ctx = traces::span_ctx(&self.tracer, SpanKind::Consumer, msg.topic());
        let span = ctx.span();

        debug!(
            trace.id = traces::trace_id(&ctx),
            span.id = traces::span_id(&ctx),
            "message received in a topic {:?}",
            msg.topic()
        );

        let dispatch = self.dispatches.get(p as usize).unwrap();

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
                Err(e)
            }
        };
    }

    pub async fn consume_blocking(&self, client: &mut MqttImplV2) -> Result<(), MqttError> {
        for topic in self.topics.clone() {
            client.subscribe(&topic, 2).await?;
        }

        while let Some(delivery) = client.stream.next().await {
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

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use async_trait::async_trait;
    use errors::mqtt::MqttError;

    #[test]
    fn test_new() {
        MqttDispatcher::new();
    }

    #[test]
    fn test_declare() {
        let mut dispatch = MqttDispatcher::new();

        let res = dispatch.declare("/some/topic".to_owned(), Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let res = dispatch.declare("".to_owned(), Arc::new(MockDispatch::new()));
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_consume() {
        let mut dispatch = MqttDispatcher::new();

        let res = dispatch.declare("/some/topic/#".to_owned(), Arc::new(MockDispatch::new()));
        assert!(res.is_ok());

        let msg = Message::new("/some/topic/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_consume_with_dispatch_return_err() {
        let mut dispatch = MqttDispatcher::new();

        let mut mock = MockDispatch::new();
        mock.set_error(MqttError::InternalError {});

        let res = dispatch.declare("/some/topic/#".to_owned(), Arc::new(mock));
        assert!(res.is_ok());

        let msg = Message::new("/some/topic/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_consume_with_unregistered_consumer() {
        let dispatch = MqttDispatcher::new();

        let msg = Message::new("/some/topic/sub", vec![], 0);

        let res = dispatch.consume(&Context::new(), &msg).await;
        assert!(res.is_err());
    }

    struct MockDispatch {
        error: Option<MqttError>,
    }

    impl MockDispatch {
        pub fn new() -> Self {
            MockDispatch { error: None }
        }

        pub fn set_error(&mut self, err: MqttError) {
            self.error = Some(err)
        }
    }

    #[async_trait]
    impl ControllerV2 for MockDispatch {
        async fn exec(
            &self,
            _ctx: &Context,
            _msgs: &[u8],
            _topic: &TopicMessage,
        ) -> Result<(), MqttError> {
            if self.error.is_some() {
                return Err(self.error.clone().unwrap());
            }

            Ok(())
        }
    }
}
