use super::{
    defs,
    topology::{AmqpTopology, ExchangeDefinition, ExchangeKind as MyExchangeKind, QueueDefinition},
    types::AmqpTracePropagator,
};
use async_trait::async_trait;
use env::Config;
use errors::amqp::AmqpError;
use events::amqp::AmqpPublishData;
use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, LongInt, LongString, ShortString},
    Channel, Connection, ConnectionProperties, Consumer, ExchangeKind,
};
#[cfg(test)]
use mockall::automock;
use opentelemetry::{global, trace::FutureExt, Context};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, error};
use uuid::Uuid;

#[cfg_attr(test, automock)]
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
    ) -> Result<(), AmqpError>;
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
        data: &AmqpPublishData,
    ) -> Result<(), AmqpError>;
    async fn install_topology<'aq>(&self, topology: &AmqpTopology<'aq>) -> Result<(), AmqpError>;
}

#[derive(Debug)]
pub struct AmqpImpl {
    conn: Box<Connection>,
    channel: Box<Channel>,
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
    ) -> Result<(), AmqpError> {
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
            .map_err(|_| AmqpError::DeclareQueueError(name.to_owned()))?;

        Ok(())
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
        data: &AmqpPublishData,
    ) -> Result<(), AmqpError> {
        let mut map = BTreeMap::new();

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(ctx, &mut AmqpTracePropagator::new(&mut map))
        });

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
            .with_context(ctx.to_owned())
            .await
            .map_err(|e| {
                error!("publish err - {:?}", e);
                AmqpError::PublishingError
            })?;

        Ok(())
    }

    async fn install_topology<'aq>(&self, topology: &AmqpTopology<'aq>) -> Result<(), AmqpError> {
        for exch in topology.exchanges.clone() {
            self.install_exchanges(&exch).await?;
        }

        for queue in topology.queues.clone() {
            self.install_queues(&queue).await?;
        }

        Ok(())
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
