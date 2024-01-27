use async_trait::async_trait;
use configs::{Configs, DynamicConfigs, Environment};
use messaging::{
    dispatcher::{Dispatcher, DispatcherDefinition},
    errors::MessagingError,
    handler::{ConsumerHandler, ConsumerMessage},
};
use opentelemetry::{
    global::{self, BoxedTracer},
    Context,
};
use rdkafka::{
    consumer::StreamConsumer,
    message::{BorrowedHeaders, Headers},
    ClientConfig, Message,
};
use std::str;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, error, warn};

use crate::otel;

pub struct KafkaDispatcher {
    consumer: Arc<StreamConsumer>,
    dispatchers: HashMap<String, Arc<dyn ConsumerHandler>>,
}

impl KafkaDispatcher {
    pub fn new<T>(cfgs: &Configs<T>) -> Result<Arc<Self>, MessagingError>
    where
        T: DynamicConfigs,
    {
        let log_level = match cfgs.app.env {
            Environment::Local | Environment::Dev => rdkafka::config::RDKafkaLogLevel::Debug,
            Environment::Staging | Environment::Prod => rdkafka::config::RDKafkaLogLevel::Info,
        };

        let consumer = match ClientConfig::new()
            .set(
                "bootstrap.servers",
                format!("{}:{}", cfgs.kafka.host, cfgs.kafka.port),
            )
            .set("client.id", cfgs.app.name.clone())
            .set("message.timeout.ms", cfgs.kafka.timeout.to_string())
            .set("security.protocol", cfgs.kafka.security_protocol.clone()) //security.protocol=SASL_PLAINTEXT or SASL_SSL
            .set("sasl.mechanism", cfgs.kafka.sasl_mechanisms.clone()) //sasl.mechanism=PLAIN
            .set("sasl.username", cfgs.kafka.user.clone())
            .set("sasl.password", cfgs.kafka.password.clone())
            .set_log_level(log_level)
            .create::<StreamConsumer>()
        {
            Ok(p) => Ok(p),
            Err(err) => {
                error!(error = err.to_string(), "failure to create kafka producer");
                Err(MessagingError::ConnectionError {})
            }
        }?;

        Ok(Arc::new(Self {
            consumer: Arc::new(consumer),
            dispatchers: HashMap::new(),
        }))
    }
}

#[async_trait]
impl Dispatcher for KafkaDispatcher {
    fn register(
        mut self,
        definition: &DispatcherDefinition,
        handler: Arc<dyn ConsumerHandler>,
    ) -> Self {
        self.dispatchers
            .insert(definition.msg_type.clone(), handler);

        self
    }

    async fn consume_blocking(&self) -> Result<(), MessagingError> {
        tokio::spawn({
            let consumer = self.consumer.clone();
            let dispatchers = self.dispatchers.clone();
            let tracer = global::tracer("kafka-consume-blocking");

            async move {
                loop {
                    let received = match consumer.recv().await {
                        Ok(m) => m,
                        Err(err) => {
                            error!(error = err.to_string(), "failure to consume message");
                            continue;
                        }
                    };

                    let topic = received.topic();

                    debug!("topic: {} - received message", topic);

                    let msg_type = match received.key() {
                        Some(k) => match str::from_utf8(k) {
                            Ok(tpy) => tpy,
                            Err(err) => {
                                error!(
                                    error = err.to_string(),
                                    topic = topic,
                                    "key conversion to utf8 error"
                                );
                                continue;
                            }
                        },
                        _ => {
                            error!(
                                topic = topic,
                                "ignoring message - message with no key (msg_type)"
                            );
                            continue;
                        }
                    };

                    if received.payload().is_none() {
                        warn!(
                            topic = topic,
                            msg_type = msg_type,
                            "ignoring msg - message with no payload"
                        );
                        continue;
                    }

                    let handler = match dispatchers.get(msg_type) {
                        Some(h) => h,
                        _ => {
                            warn!(
                                topic = topic,
                                msg_type = msg_type,
                                "ignoring message - there is no handler registered for this msg_type",
                            );

                            continue;
                        }
                    };

                    let (ctx, headers) = explode(topic, msg_type, &tracer, received.headers());
                    let consumer_msg =
                        ConsumerMessage::new(topic, msg_type, received.payload().unwrap(), headers);

                    match handler.exec(&ctx, &consumer_msg).await {
                        Err(err) => {
                            error!(
                                error = err.to_string(),
                                topic = topic,
                                msg_type = msg_type,
                                "error whiling processing message"
                            );
                            continue;
                        }
                        _ => {
                            debug!(
                                topic = topic,
                                msg_type = msg_type,
                                "message processed succeffly"
                            )
                        }
                    };
                }
            }
        });

        Ok(())
    }
}

fn explode(
    topic: &str,
    msg_type: &str,
    tracer: &BoxedTracer,
    kafka_headers: Option<&BorrowedHeaders>,
) -> (Context, Option<HashMap<String, String>>) {
    let Some(headers) = kafka_headers else {
        return (otel::new_ctx(topic, msg_type, tracer), None);
    };

    let ctx = match otel::extract_context(headers) {
        Ok(ctx) => ctx,
        _ => otel::new_ctx(topic, msg_type, tracer),
    };

    let mut map = HashMap::with_capacity(headers.count());

    for h in headers.iter() {
        let value = match std::str::from_utf8(h.value.unwrap()) {
            Ok(v) => v,
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "failure to conversion the key to utf8 error"
                );
                continue;
            }
        };

        map.insert(h.key.into(), value.into());
    }

    (ctx, Some(map))
}
