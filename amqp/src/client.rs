use super::{
    defs,
    topology::{
        AmqpTopology, ConsumerDefinition, ConsumerHandler, ExchangeDefinition,
        ExchangeKind as MyExchangeKind, QueueDefinition,
    },
    types::{Metadata, PublishData},
};
use async_trait::async_trait;
use env::Config;
use errors::amqp::AmqpError;
use lapin::{
    message::Delivery,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, LongInt, LongString, ShortString},
    Channel, Connection, ConnectionProperties, Consumer, ExchangeKind, Queue,
};
use log::{debug, error, warn};
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{FutureExt, Span, StatusCode},
    Context,
};
use otel;
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[async_trait]
pub trait Amqp {
    fn channel(&self) -> &Channel;
    fn connection(&self) -> &Connection;
    async fn declare_queue(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        exclusive: bool,
    ) -> Result<Queue, AmqpError>;
    async fn declare_exchange(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        internal: bool,
    ) -> Result<(), AmqpError>;
    async fn binding_exchange_queue(
        &self,
        exch: &str,
        queue: &str,
        key: &str,
    ) -> Result<(), AmqpError>;
    async fn consumer(&self, queue: &str, tag: &str) -> Result<Consumer, AmqpError>;
    async fn publish(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        data: &PublishData,
    ) -> Result<(), AmqpError>;
    async fn install_topology(&self, topology: &AmqpTopology) -> Result<(), AmqpError>;
    async fn consume<'c>(
        &self,
        def: &'c ConsumerDefinition,
        handler: Arc<dyn ConsumerHandler + Send + Sync>,
        delivery: &'c Delivery,
    ) -> Result<(), AmqpError>;
}

#[derive(Debug)]
pub struct AmqpImpl {
    conn: Box<Connection>,
    channel: Box<Channel>,
    tracer: BoxedTracer,
}

impl AmqpImpl {
    pub async fn new(cfg: &Config) -> Result<Arc<dyn Amqp + Send + Sync>, AmqpError> {
        debug!("creating amqp connection...");
        let options = ConnectionProperties::default()
            .with_connection_name(LongString::from(cfg.app_name.clone()));

        let uri = &cfg.amqp_uri();
        let conn = Connection::connect(uri, options)
            .await
            .map_err(|_| AmqpError::ConnectionError {})?;
        debug!("amqp connected");

        debug!("creating amqp channel...");
        let channel = conn
            .create_channel()
            .await
            .map_err(|_| AmqpError::ChannelError {})?;
        debug!("channel created");

        Ok(Arc::new(AmqpImpl {
            conn: Box::new(conn),
            channel: Box::new(channel),
            tracer: global::tracer("amqp"),
        }))
    }
}

#[async_trait]
impl Amqp for AmqpImpl {
    fn channel(&self) -> &Channel {
        &self.channel
    }

    fn connection(&self) -> &Connection {
        &self.conn
    }

    async fn declare_queue(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        exclusive: bool,
    ) -> Result<Queue, AmqpError> {
        self.channel
            .queue_declare(
                name,
                QueueDeclareOptions {
                    auto_delete: delete,
                    durable,
                    exclusive,
                    nowait: false,
                    passive: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(name.to_owned()))
    }

    async fn declare_exchange(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        internal: bool,
    ) -> Result<(), AmqpError> {
        self.channel
            .exchange_declare(
                name,
                ExchangeKind::Direct,
                ExchangeDeclareOptions {
                    auto_delete: delete,
                    durable,
                    internal,
                    nowait: false,
                    passive: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::DeclareExchangeError(name.to_owned()))
    }

    async fn binding_exchange_queue(
        &self,
        exch: &str,
        queue: &str,
        key: &str,
    ) -> Result<(), AmqpError> {
        self.channel
            .queue_bind(
                queue,
                exch,
                key,
                QueueBindOptions { nowait: false },
                FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::BindingExchangeToQueueError(exch.to_owned(), queue.to_owned()))
    }

    async fn consumer(&self, queue: &str, tag: &str) -> Result<Consumer, AmqpError> {
        self.channel
            .basic_consume(
                queue,
                tag,
                BasicConsumeOptions {
                    exclusive: false,
                    no_ack: false,
                    no_local: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::BindingConsumerError(tag.to_owned()))
    }

    async fn publish(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        data: &PublishData,
    ) -> Result<(), AmqpError> {
        let cx = otel::tracing::ctx_from_ctx(&self.tracer, ctx, "amqp publishing");

        let mut map = BTreeMap::new();
        map.insert(
            ShortString::from(defs::AMQP_HEADERS_OTEL_TRACEPARENT),
            AMQPValue::LongString(LongString::from(otel::amqp::Traceparent::string_from_ctx(
                &cx,
            ))),
        );

        self.channel
            .basic_publish(
                exchange,
                key,
                BasicPublishOptions {
                    immediate: false,
                    mandatory: false,
                },
                &data.payload,
                AMQPProperties::default()
                    .with_content_type(ShortString::from(defs::AMQP_JSON_CONTENT_TYPE))
                    .with_kind(ShortString::from(data.clone().msg_type))
                    .with_message_id(ShortString::from(Uuid::new_v4().to_string()))
                    .with_headers(FieldTable::from(map)),
            )
            .with_context(cx)
            .await
            .map_err(|e| {
                error!("publish err - {:?}", e);
                AmqpError::PublishingError
            })?;

        Ok(())
    }

    async fn install_topology(&self, topology: &AmqpTopology) -> Result<(), AmqpError> {
        for exch in topology.exchanges.clone() {
            self.install_exchanges(&exch).await?;
        }

        for queue in topology.queues.clone() {
            self.install_queues(&queue).await?;
        }

        Ok(())
    }

    async fn consume<'c>(
        &self,
        def: &'c ConsumerDefinition<'c>,
        handler: Arc<dyn ConsumerHandler + Send + Sync>,
        delivery: &'c Delivery,
    ) -> Result<(), AmqpError> {
        let header = match delivery.properties.headers() {
            Some(val) => val.to_owned(),
            None => FieldTable::default(),
        };

        let metadata = Metadata::extract(&header);

        debug!(
            "queue: {} - consumer: {} - received message: {}",
            def.name, def.queue, metadata.msg_type
        );

        if metadata.msg_type.is_empty() || metadata.msg_type != def.msg_type.to_string() {
            match delivery
                .nack(BasicNackOptions {
                    multiple: false,
                    requeue: true,
                })
                .await
            {
                Ok(_) => {}
                _ => error!("error whiling ack msg"),
            };
            return Ok(());
        }

        let (ctx, mut span) =
            otel::amqp::get_span(&self.tracer, &metadata.traceparent, &metadata.msg_type);

        match handler
            .exec(&ctx, delivery.data.as_slice())
            .with_context(ctx.clone())
            .await
        {
            Ok(_) => match delivery.ack(BasicAckOptions { multiple: false }).await {
                Ok(_) => {
                    span.set_status(StatusCode::Ok, "success".to_owned());
                    return Ok(());
                }
                _ => {
                    error!("error whiling ack msg");
                    span.set_status(StatusCode::Error, "error to ack msg".to_owned());
                    return Err(AmqpError::AckMessageError {});
                }
            },
            _ if def.with_retry => {
                if metadata.count <= def.retries {
                    warn!("error whiling handling msg, requeuing for latter");
                    match delivery
                        .nack(BasicNackOptions {
                            multiple: false,
                            requeue: false,
                        })
                        .await
                    {
                        Ok(_) => return Ok(()),
                        _ => {
                            error!("error whiling requeuing");
                            span.set_status(StatusCode::Error, "error to requeuing msg".to_owned());
                            return Err(AmqpError::RequeuingMessageError {});
                        }
                    }
                } else {
                    error!("too many attempts, sending to dlq");
                    match self
                        .channel
                        .basic_publish(
                            "",
                            def.dlq_name,
                            BasicPublishOptions::default(),
                            &delivery.data,
                            delivery.properties.clone(),
                        )
                        .await
                    {
                        Ok(_) => {
                            match delivery.ack(BasicAckOptions { multiple: false }).await {
                                Ok(_) => return Ok(()),
                                _ => {
                                    error!("error whiling ack msg to default queue");
                                    span.set_status(
                                        StatusCode::Error,
                                        "msg was sent to dlq".to_owned(),
                                    );
                                    return Err(AmqpError::AckMessageError {});
                                }
                            };
                        }
                        _ => {
                            error!("error whiling sending to dlq");
                            span.set_status(StatusCode::Error, "msg was sent to dlq".to_owned());
                            return Err(AmqpError::PublishingToDQLError {});
                        }
                    };
                }
            }
            _ => {
                match delivery
                    .nack(BasicNackOptions {
                        multiple: false,
                        requeue: false,
                    })
                    .await
                {
                    Ok(_) => return Ok(()),
                    _ => {
                        error!("error whiling nack msg");
                        span.set_status(StatusCode::Error, "error to nack msg".to_owned());
                        return Err(AmqpError::NackMessageError {});
                    }
                }
            }
        }
    }
}

impl AmqpImpl {
    async fn install_exchanges<'i>(
        &self,
        exch: &'i ExchangeDefinition<'i>,
    ) -> Result<(), AmqpError> {
        debug!("creating exchange: {}", exch.name);

        self.channel
            .exchange_declare(
                exch.name,
                MyExchangeKind::map(exch.kind.clone()),
                ExchangeDeclareOptions {
                    auto_delete: false,
                    durable: true,
                    internal: false,
                    nowait: false,
                    passive: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::DeclareExchangeError(exch.name.to_owned()))?;

        debug!("exchange: {} was created", exch.name);

        Ok(())
    }
}

impl AmqpImpl {
    async fn install_queues<'i>(&self, def: &'i QueueDefinition<'i>) -> Result<(), AmqpError> {
        debug!("creating and binding queue: {}", def.name);

        let queue_map = self.install_retry(def).await?;
        let queue_map = self.install_dlq(def, queue_map).await?;

        self.channel
            .queue_declare(
                def.name,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::from(queue_map),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(def.name.to_owned()))?;

        for bind in def.clone().bindings {
            self.channel
                .queue_bind(
                    bind.queue,
                    bind.exchange,
                    bind.routing_key,
                    QueueBindOptions { nowait: false },
                    FieldTable::default(),
                )
                .await
                .map_err(|_| {
                    AmqpError::BindingExchangeToQueueError(
                        bind.exchange.to_owned(),
                        bind.queue.to_owned(),
                    )
                })?;
        }

        debug!("queue: {} was created and bonded", def.name);

        Ok(())
    }

    async fn install_retry<'i>(
        &self,
        def: &'i QueueDefinition<'i>,
    ) -> Result<BTreeMap<ShortString, AMQPValue>, AmqpError> {
        if !def.with_retry {
            return Ok(BTreeMap::new());
        }

        debug!("creating retry...");
        let mut retry_map = BTreeMap::new();
        retry_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
            AMQPValue::LongString(LongString::from("")),
        );
        retry_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
            AMQPValue::LongString(LongString::from(def.name)),
        );
        retry_map.insert(
            ShortString::from(defs::AMQP_HEADERS_MESSAGE_TTL),
            AMQPValue::LongInt(LongInt::from(def.retry_ttl.unwrap())),
        );

        let name = self.retry_name(def.name);
        self.channel
            .queue_declare(
                &name,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::from(retry_map),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(name.clone()))?;

        let mut queue_map = BTreeMap::new();
        queue_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
            AMQPValue::LongString(LongString::from("")),
        );

        queue_map.insert(
            ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
            AMQPValue::LongString(LongString::from(name)),
        );
        debug!("retry created");

        Ok(queue_map)
    }

    async fn install_dlq<'i>(
        &self,
        def: &'i QueueDefinition<'i>,
        queue_map_from_retry: BTreeMap<ShortString, AMQPValue>,
    ) -> Result<BTreeMap<ShortString, AMQPValue>, AmqpError> {
        if !def.with_dlq && !def.with_retry {
            return Ok(BTreeMap::new());
        }

        debug!("creating DLQ...");
        let mut queue_map = queue_map_from_retry;
        let name = self.dlq_name(def.name);

        if !def.with_retry {
            queue_map.insert(
                ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
                AMQPValue::LongString(LongString::from("")),
            );

            queue_map.insert(
                ShortString::from(defs::AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
                AMQPValue::LongString(LongString::from(name.clone())),
            );
        }

        self.channel
            .queue_declare(
                &name,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(name))?;
        debug!("DLQ created");

        Ok(queue_map)
    }

    fn retry_name(&self, queue: &str) -> String {
        format!("{}-retry", queue)
    }

    fn _retry_key(&self, queue: &str) -> String {
        format!("{}-retry-key", queue)
    }

    fn dlq_name(&self, queue: &str) -> String {
        format!("{}-dlq", queue)
    }

    fn _dlq_key(&self, queue: &str) -> String {
        format!("{}-dlq-key", queue)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn consume_successfully() {}
}
