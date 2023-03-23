use super::defs;
use crate::errors::AmqpError;
use lapin::{
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, LongInt, ShortString},
};
use opentelemetry::{
    global::{BoxedSpan, BoxedTracer},
    propagation::{Extractor, Injector},
    trace::{SpanKind, Tracer},
    Context,
};
use serde::Serialize;
use std::{borrow::Cow, collections::BTreeMap};
use tracing::error;

#[derive(Debug, Copy, Clone, Default)]
pub struct PublishParams {
    x_delay: Option<i32>,
}

impl PublishParams {
    pub fn to_btree(&self) -> BTreeMap<ShortString, AMQPValue> {
        let mut map: BTreeMap<ShortString, AMQPValue> = BTreeMap::new();

        if let Some(delay) = self.x_delay {
            map.insert(
                ShortString::from("x-delay"),
                AMQPValue::LongInt(LongInt::from(delay)),
            );
        }

        map
    }

    pub fn new() -> PublishParams {
        PublishParams::default()
    }

    pub fn delay(mut self, d: i32) -> Self {
        self.x_delay = Some(d);
        self
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub msg_type: String,
    pub count: i64,
}

impl Metadata {
    pub fn extract(props: &AMQPProperties) -> Metadata {
        let headers = match props.headers() {
            Some(val) => val.to_owned(),
            None => FieldTable::default(),
        };

        let count = match headers.inner().get(defs::AMQP_HEADERS_X_DEATH) {
            Some(value) => match value.as_array() {
                Some(arr) => match arr.as_slice().get(0) {
                    Some(value) => match value.as_field_table() {
                        Some(table) => match table.inner().get(defs::AMQP_HEADERS_COUNT) {
                            Some(value) => match value.as_long_long_int() {
                                Some(long) => long,
                                _ => 0,
                            },
                            _ => 0,
                        },
                        _ => 0,
                    },
                    _ => 0,
                },
                _ => 0,
            },
            _ => 0,
        };

        let msg_type = match props.kind() {
            Some(value) => value.to_string(),
            _ => "".to_owned(),
        };

        Metadata { msg_type, count }
    }
}

pub trait AmqpMessage {
    fn get_type(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct AmqpPayload {
    pub payload: Box<[u8]>,
    pub msg_type: String,
}

impl AmqpPayload {
    pub fn new<T>(payload: &T) -> Result<Self, AmqpError>
    where
        T: AmqpMessage + Serialize,
    {
        let serialized = serde_json::to_vec::<T>(&payload)
            .map_err(|_| AmqpError::ParsePayloadError {})?
            .into_boxed_slice();

        Ok(AmqpPayload {
            msg_type: payload.get_type(),
            payload: serialized,
        })
    }
}

pub(crate) struct AmqpTracePropagator<'a> {
    headers: &'a mut BTreeMap<ShortString, AMQPValue>,
}

impl<'a> AmqpTracePropagator<'a> {
    pub(crate) fn new(headers: &'a mut BTreeMap<ShortString, AMQPValue>) -> Self {
        Self { headers }
    }
}

impl<'a> Injector for AmqpTracePropagator<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(
            key.to_lowercase().into(),
            AMQPValue::LongString(value.into()),
        );
    }
}

impl<'a> Extractor for AmqpTracePropagator<'a> {
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
        propagator.extract(&AmqpTracePropagator::new(
            &mut props.headers().clone().unwrap_or_default().inner().clone(),
        ))
    });

    let span = tracer
        .span_builder(Cow::from(name.to_owned()))
        .with_kind(SpanKind::Consumer)
        .start_with_context(tracer, &ctx);

    (ctx, span)
}

#[cfg(test)]
mod tests {
    // use std::collections::BTreeMap;

    // use super::*;
    // use lapin::types::{AMQPValue, FieldArray, FieldTable, LongLongInt, LongString, ShortString};

    #[test]
    fn test_metadata_extract_successfully() {
        // let mut count = BTreeMap::new();
        // count.insert(ShortString::from("count"), AMQPValue::LongLongInt(10));

        // let mut props = AMQPProperties

        // let mut metadata = BTreeMap::new();
        // metadata.insert(
        //     ShortString::from("x-death"),
        //     AMQPValue::FieldArray(FieldArray::from(vec![AMQPValue::FieldTable(
        //         FieldTable::from(count),
        //     )])),
        // );

        // metadata.insert(
        //     ShortString::from("traceparent"),
        //     AMQPValue::LongString(LongString::from("traceparent")),
        // );

        // metadata.insert(
        //     ShortString::from("type"),
        //     AMQPValue::LongString(LongString::from("msg_type")),
        // );

        // let re = Metadata::extract(&FieldTable::from(metadata));
        // assert_eq!(re.count, 10);
        // assert_eq!(re.traceparent, "traceparent");
        // assert_eq!(re.msg_type, "msg_type");
    }

    #[test]
    fn test_metadata_extract_wrong() {
        // let mut metadata = BTreeMap::new();

        // let re = Metadata::extract(&FieldTable::from(metadata.clone()));
        // assert_eq!(re.count, 0);
        // assert_eq!(re.traceparent, "");

        // let mut count = BTreeMap::new();
        // count.insert(ShortString::from("c"), AMQPValue::LongLongInt(10));
        // metadata.insert(
        //     ShortString::from("x-death"),
        //     AMQPValue::FieldArray(FieldArray::from(vec![AMQPValue::FieldTable(
        //         FieldTable::from(count),
        //     )])),
        // );
        // metadata.insert(
        //     ShortString::from("traceparent"),
        //     AMQPValue::LongLongInt(LongLongInt::from(10)),
        // );
        // metadata.insert(
        //     ShortString::from("type"),
        //     AMQPValue::LongLongInt(LongLongInt::from(10)),
        // );
        // let re = Metadata::extract(&FieldTable::from(metadata.clone()));
        // assert_eq!(re.count, 0);
        // assert_eq!(re.traceparent, "");
    }
}
