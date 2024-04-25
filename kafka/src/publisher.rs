use async_trait::async_trait;
use configs::{Configs, DynamicConfigs, Environment};
use messaging::{
    errors::MessagingError,
    publisher::{HeaderValues, PublishMessage, Publisher},
};
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{Status, TraceContextExt},
    Context,
};
use rdkafka::{
    message::{Header, OwnedHeaders, ToBytes},
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::error;

use crate::otel;

/// LongInt
pub const PARTITION_HEADER_KEY: &str = "kafka-partition";

/// LongLongInt
pub const TIMESTAMP_HEADER_KEY: &str = "kafka-timestamp";

/// LongLongUint
pub const QUEUE_TIMEOUT_KEY: &str = "kafka-queue-timeout";

pub struct KafkaPublisher {
    producer: Arc<FutureProducer>,
    tracer: BoxedTracer,
}

impl KafkaPublisher {
    pub fn new<T>(cfgs: &Configs<T>) -> Result<Arc<Self>, MessagingError>
    where
        T: DynamicConfigs,
    {
        let log_level = match cfgs.app.env {
            Environment::Local | Environment::Dev => rdkafka::config::RDKafkaLogLevel::Debug,
            Environment::Staging | Environment::Prod => rdkafka::config::RDKafkaLogLevel::Info,
        };

        let producer = match ClientConfig::new()
            .set(
                "bootstrap.servers",
                format!("{}:{}", cfgs.kafka.host, cfgs.kafka.port),
            )
            .set("client.id", cfgs.app.name.clone())
            .set("acks", "1")
            .set("message.timeout.ms", cfgs.kafka.timeout.to_string())
            .set("security.protocol", cfgs.kafka.security_protocol.clone()) //security.protocol=SASL_PLAINTEXT or SASL_SSL
            .set("sasl.mechanism", cfgs.kafka.sasl_mechanisms.clone()) //sasl.mechanism=PLAIN
            .set("sasl.username", cfgs.kafka.user.clone())
            .set("sasl.password", cfgs.kafka.password.clone())
            .set_log_level(log_level)
            .create::<FutureProducer>()
        {
            Ok(p) => Ok(p),
            Err(err) => {
                error!(error = err.to_string(), "failure to create kafka producer");
                Err(MessagingError::ConnectionError {})
            }
        }?;

        Ok(Arc::new(Self {
            producer: Arc::new(producer),
            tracer: global::tracer("kafka-publisher"),
        }))
    }
}

#[async_trait]
impl Publisher for KafkaPublisher {
    async fn publish(&self, ctx: &Context, msg: &PublishMessage) -> Result<(), MessagingError> {
        let (partition, timestamp, queue_timeout) = self.publish_configs(&msg.headers);
        let headers = self.headers(ctx, msg);

        let mut record = FutureRecord::to(&msg.to)
            .key(&msg.key)
            .timestamp(timestamp)
            .headers(headers)
            .payload(msg.data.to_bytes());

        if partition.is_some() {
            record.partition = partition;
        }

        let span = ctx.span();

        match self.producer.send(record, queue_timeout).await {
            Err((err, _)) => {
                error!(error = err.to_string(), "failure to publish");

                span.set_status(Status::error(err.to_string()));
                span.record_error(&MessagingError::PublisherError {});

                Err(MessagingError::PublisherError {})
            }
            _ => Ok(()),
        }
    }
}

impl KafkaPublisher {
    fn publish_configs(
        &self,
        headers: &Option<HashMap<String, HeaderValues>>,
    ) -> (Option<i32>, i64, Duration) {
        if headers.is_none() {
            return (None, now(), Duration::from_secs(0));
        }

        let headers = headers.as_ref().unwrap();

        let partition = match headers.get(PARTITION_HEADER_KEY) {
            Some(v) => {
                if let HeaderValues::LongInt(p) = v {
                    Some(p.to_owned())
                } else {
                    None
                }
            }
            None => None,
        };

        let timestamp = match headers.get(TIMESTAMP_HEADER_KEY) {
            Some(v) => {
                if let HeaderValues::LongLongInt(t) = v {
                    t.to_owned()
                } else {
                    now()
                }
            }
            None => now(),
        };

        let queue_timeout = match headers.get(QUEUE_TIMEOUT_KEY) {
            Some(v) => {
                if let HeaderValues::LongLongUint(t) = v {
                    Duration::from_millis(t.to_owned())
                } else {
                    Duration::from_secs(0)
                }
            }
            None => Duration::from_secs(0),
        };

        (partition, timestamp, queue_timeout)
    }

    fn headers(&self, ctx: &Context, msg: &PublishMessage) -> OwnedHeaders {
        let Some(headers) = msg.headers.clone() else {
            return OwnedHeaders::new();
        };

        let mut kafka_headers = OwnedHeaders::new();

        for (key, value) in headers {
            if key.eq(PARTITION_HEADER_KEY)
                || key.eq(TIMESTAMP_HEADER_KEY)
                || key.eq(QUEUE_TIMEOUT_KEY)
            {
                continue;
            }

            let vv: String = value.into();

            kafka_headers = kafka_headers.insert(Header {
                key: &key,
                value: Some(&vv),
            })
        }

        otel::inject_context(ctx, &msg.to, &msg.msg_type, &self.tracer, kafka_headers)
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
