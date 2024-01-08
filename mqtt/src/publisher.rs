use async_trait::async_trait;
use messaging::{
    errors::MessagingError,
    publisher::{HeaderValues, PublishMessage, Publisher},
};
use opentelemetry::{
    trace::{Status, TraceContextExt},
    Context,
};
use paho_mqtt::{AsyncClient, Message};
use std::{borrow::Cow, sync::Arc};
use tracing::error;

pub struct MQTTPublisher {
    conn: Arc<AsyncClient>,
}

impl MQTTPublisher {
    pub fn new(conn: Arc<AsyncClient>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Publisher for MQTTPublisher {
    async fn publish(&self, ctx: &Context, infos: &PublishMessage) -> Result<(), MessagingError> {
        let span = ctx.span();

        let mut qos: i32 = 0;

        if let Some(headers) = &infos.headers {
            if let Some(custom_qos) = headers.get("qos") {
                if let HeaderValues::Int(custom) = custom_qos {
                    qos = custom.to_owned() as i32;
                }

                if let HeaderValues::LongInt(custom) = custom_qos {
                    qos = custom.to_owned();
                }
            }
        }

        match self
            .conn
            .publish(Message::new(infos.to.clone(), infos.data.clone(), qos))
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error to publish message");

                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to publish"),
                });

                Err(MessagingError::PublisherError {})
            }
            _ => {
                span.set_status(Status::Ok);
                Ok(())
            }
        }
    }
}
