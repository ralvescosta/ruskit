use lapin::{
    protocol::basic::AMQPProperties,
    types::{AMQPValue, ShortString},
};
use opentelemetry::{
    global::{BoxedSpan, BoxedTracer},
    propagation::{Extractor, Injector},
    trace::{SpanKind, Tracer},
    Context,
};
use std::{borrow::Cow, collections::BTreeMap};
use tracing::error;

pub(crate) struct RabbitMQTracePropagator<'a> {
    headers: &'a mut BTreeMap<ShortString, AMQPValue>,
}

impl<'a> RabbitMQTracePropagator<'a> {
    pub(crate) fn new(headers: &'a mut BTreeMap<ShortString, AMQPValue>) -> Self {
        Self { headers }
    }
}

impl<'a> Injector for RabbitMQTracePropagator<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(
            key.to_lowercase().into(),
            AMQPValue::LongString(value.into()),
        );
    }
}

impl<'a> Extractor for RabbitMQTracePropagator<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|header_value| {
            if let AMQPValue::LongString(header_value) = header_value {
                std::str::from_utf8(header_value.as_bytes())
                    .map_err(|e| error!("Error decoding header value {:?}", e))
                    .ok()
            } else {
                None
            }
        })
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|header| header.as_str()).collect()
    }
}

pub fn new_span(props: &AMQPProperties, tracer: &BoxedTracer, name: &str) -> (Context, BoxedSpan) {
    let ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&RabbitMQTracePropagator::new(
            &mut props.headers().clone().unwrap_or_default().inner().clone(),
        ))
    });

    let span = tracer
        .span_builder(Cow::from(name.to_owned()))
        .with_kind(SpanKind::Consumer)
        .start_with_context(tracer, &ctx);

    (ctx, span)
}
