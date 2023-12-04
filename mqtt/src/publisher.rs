use crate::{errors::MQTTError, payload::Payload};
use async_trait::async_trait;
use opentelemetry::Context;
use paho_mqtt::{AsyncClient, Message};
use std::sync::Arc;
use tracing::error;

#[cfg(test)]
use mockall::*;
#[cfg(feature = "mocks")]
use mockall::*;

#[cfg_attr(test, automock)]
#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait Publisher {
    async fn publish(
        &self,
        ctx: &Context,
        topic: &str,
        payload: &Payload,
        qos: i32,
    ) -> Result<(), MQTTError>;
}

pub struct MQTTPublisher {
    conn: Arc<AsyncClient>,
}

#[async_trait]
impl Publisher for MQTTPublisher {
    async fn publish(
        &self,
        _ctx: &Context,
        topic: &str,
        payload: &Payload,
        qos: i32,
    ) -> Result<(), MQTTError> {
        match self
            .conn
            .publish(Message::new(topic, payload.0.clone(), qos))
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error to publish message");
                Err(MQTTError::PublishingError {})
            }
            _ => Ok(()),
        }
    }
}
