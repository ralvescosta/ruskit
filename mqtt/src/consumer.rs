use std::sync::Arc;

use errors::mqtt::MqttError;
use events::mqtt::TopicMessage;
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{SpanKind, TraceContextExt},
    Context,
};
use paho_mqtt::{Message, Topic, TopicFilter};
use tracing::{debug, error, warn};

use crate::types::ControllerV2;

pub struct MqttDispatches {
    topics: Vec<String>,
    dispatches: Vec<Arc<dyn ControllerV2 + Sync + Send>>,
    tracer: BoxedTracer,
}

impl MqttDispatches {
    pub fn new() -> Self {
        MqttDispatches {
            topics: vec![],
            dispatches: vec![],
            tracer: global::tracer("mqtt_consumer"),
        }
    }

    pub fn declare(&mut self, topic: String, handler: Arc<dyn ControllerV2 + Send + Sync>) {
        self.topics.push(topic);
        self.dispatches.push(handler);
    }

    pub async fn consume(&self, ctx: Context, msg: Message) -> Result<(), MqttError> {
        let filter = TopicFilter::new(msg.topic()).map_err(|e| {
            error!(
                error = e.to_string(),
                trace.id = traces::trace_id(&ctx),
                span.id = traces::span_id(&ctx),
                "error to create mqtt topic filter",
            );
            MqttError::InternalError {}
        })?;

        let mut p = -1;
        for (i, tp) in self.topics.clone().into_iter().enumerate() {
            if !filter.is_match(&tp) {
                continue;
            }
            p = i as i8;
            break;
        }

        if p == -1 {
            warn!(
                trace.id = traces::trace_id(&ctx),
                span.id = traces::span_id(&ctx),
                "cant find dispatch for this topic"
            );
            return Err(MqttError::UnknownMessageKindError {});
        }

        let metadata =
            TopicMessage::new(msg.topic()).map_err(|_| MqttError::UnformattedTopicError {})?;

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
}
