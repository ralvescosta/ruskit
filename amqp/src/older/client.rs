use super::{
    defs,
    topology::ExchangeKind as MyExchangeKind,
    types::{AmqpPayload, AmqpTracePropagator},
};
use crate::{errors::AmqpError, types::PublishParams};
use async_trait::async_trait;
use env::{Configs, DynamicConfig};
use lapin::{
    message::BasicGetMessage,
    options::{
        BasicConsumeOptions, BasicGetOptions, BasicPublishOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, LongString, ShortString},
    Channel, Connection, ConnectionProperties, Consumer,
};
use opentelemetry::{global, trace::FutureExt, Context};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, error};
use uuid::Uuid;

#[async_trait]
pub trait Amqp {
    fn channel(&self) -> Arc<Channel>;
    fn connection(&self) -> Arc<Connection>;
    async fn declare_queue(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        exclusive: bool,
        configs: &FieldTable,
    ) -> Result<(), AmqpError>;
    async fn declare_exchange(
        &self,
        name: &str,
        kind: MyExchangeKind,
        delete: bool,
        durable: bool,
        internal: bool,
        params: Option<BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError>;
    async fn binding_exchange_queue(
        &self,
        exch: &str,
        queue: &str,
        key: &str,
    ) -> Result<(), AmqpError>;
    async fn get(&self, queue: &str) -> Result<Option<BasicGetMessage>, AmqpError>;
    async fn consumer(&self, queue: &str, tag: &str) -> Result<Consumer, AmqpError>;
    async fn publish(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        data: &AmqpPayload,
        params: &PublishParams,
    ) -> Result<(), AmqpError>;
}

#[derive(Debug)]
pub struct AmqpImpl {
    conn: Arc<Connection>,
    channel: Arc<Channel>,
}

impl AmqpImpl {
    pub async fn new<T>(cfg: &Configs<T>) -> Result<Arc<dyn Amqp + Send + Sync>, AmqpError>
    where
        T: DynamicConfig,
    {
        debug!("creating amqp connection...");
        let options = ConnectionProperties::default()
            .with_connection_name(LongString::from(cfg.app.name.clone()));

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
            conn: Arc::new(conn),
            channel: Arc::new(channel),
        }))
    }
}

#[async_trait]
impl Amqp for AmqpImpl {
    fn channel(&self) -> Arc<Channel> {
        self.channel.clone()
    }

    fn connection(&self) -> Arc<Connection> {
        self.conn.clone()
    }

    async fn declare_queue(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        exclusive: bool,
        configs: &FieldTable,
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
                configs.to_owned(),
            )
            .await
            .map_err(|_| AmqpError::DeclareQueueError(name.to_owned()))?;

        Ok(())
    }

    async fn declare_exchange(
        &self,
        name: &str,
        kind: MyExchangeKind,
        delete: bool,
        durable: bool,
        internal: bool,
        params: Option<BTreeMap<ShortString, AMQPValue>>,
    ) -> Result<(), AmqpError> {
        let params = params.unwrap_or_default();

        self.channel
            .exchange_declare(
                name,
                MyExchangeKind::try_into(kind),
                ExchangeDeclareOptions {
                    auto_delete: delete,
                    durable,
                    internal,
                    nowait: false,
                    passive: false,
                },
                FieldTable::from(params),
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

    async fn get(&self, queue: &str) -> Result<Option<BasicGetMessage>, AmqpError> {
        let r = self
            .channel
            .basic_get(queue, BasicGetOptions { no_ack: false })
            .await
            .map_err(|_| AmqpError::BindingConsumerError(queue.to_owned()))?;

        Ok(r)
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
        data: &AmqpPayload,
        params: &PublishParams,
    ) -> Result<(), AmqpError> {
        let mut params = params.to_btree();

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(ctx, &mut AmqpTracePropagator::new(&mut params))
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
                    .with_headers(FieldTable::from(params)),
            )
            .with_context(ctx.to_owned())
            .await
            .map_err(|e| {
                error!("publish err - {:?}", e);
                AmqpError::PublishingError
            })?;

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn consume_successfully() {}
}
