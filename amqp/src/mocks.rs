use super::{
    topology::ExchangeKind as MyExchangeKind,
    types::{AmqpPayload, PublishParams},
};
use crate::client::Amqp;
use async_trait::async_trait;
use errors::amqp::AmqpError;
use lapin::{
    types::{AMQPValue, FieldTable, ShortString},
    Channel, Connection, Consumer,
};
use mockall::*;
use opentelemetry::Context;
use std::{collections::BTreeMap, sync::Arc};

mock! {
  pub AmqpImpl {}

  #[async_trait]
  impl Amqp for AmqpImpl {
    fn channel(&self) -> Arc<Channel> {
      todo!()
    }

    fn connection(&self) -> Arc<Connection> {
      todo!()
    }

    async fn declare_queue(
        &self,
        name: &str,
        delete: bool,
        durable: bool,
        exclusive: bool,
        configs: &FieldTable,
    ) -> Result<(), AmqpError> {
      todo!()
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
      todo!()
    }

    async fn binding_exchange_queue(
        &self,
        exch: &str,
        queue: &str,
        key: &str,
    ) -> Result<(), AmqpError> {
      todo!()
    }

    async fn consumer(&self, queue: &str, tag: &str) -> Result<Consumer, AmqpError> {
      todo!()
    }

    async fn publish(
        &self,
        ctx: &Context,
        exchange: &str,
        key: &str,
        data: &AmqpPayload,
        params: &PublishParams,
    ) -> Result<(), AmqpError> {
      todo!()
    }
  }
}
