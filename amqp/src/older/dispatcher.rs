use crate::{
    client::Amqp,
    consumer::consume,
    errors::AmqpError,
    topology::{ConsumerHandler, QueueDefinition},
};
use futures_util::{future::join_all, StreamExt};
use opentelemetry::global;
use std::sync::Arc;
use tokio::task::JoinError;
use tracing::error;

pub struct Dispatcher {
    amqp: Arc<dyn Amqp + Send + Sync>,
    pub(crate) queues: Vec<QueueDefinition>,
    pub(crate) msgs_types: Vec<String>,
    pub(crate) handlers: Vec<Arc<dyn ConsumerHandler + Send + Sync>>,
}

impl Dispatcher {
    pub fn new(amqp: Arc<dyn Amqp + Send + Sync>) -> Dispatcher {
        Dispatcher {
            amqp,
            queues: vec![],
            msgs_types: vec![],
            handlers: vec![],
        }
    }

    pub fn declare(
        &mut self,
        queue: QueueDefinition,
        msg_type: String,
        handler: Arc<dyn ConsumerHandler + Send + Sync>,
    ) -> Result<(), AmqpError> {
        if msg_type.is_empty() {
            return Err(AmqpError::ConsumerDeclarationError {});
        }

        self.queues.push(queue);
        self.msgs_types.push(msg_type);
        self.handlers.push(handler);

        Ok(())
    }

    pub async fn consume_blocking(&self) -> Vec<Result<(), JoinError>> {
        let mut spawns = vec![];

        for i in 0..self.queues.len() {
            spawns.push(tokio::spawn({
                let m_amqp = self.amqp.clone();
                let msgs_allowed = self.msgs_types.clone();

                let queue = self.queues[i].clone();
                let msg_type = self.msgs_types[i].clone();
                let handlers = self.handlers.clone();

                let mut consumer = m_amqp
                    .consumer(&queue.name, &msg_type)
                    .await
                    .expect("unexpected error while creating the consumer");

                async move {
                    while let Some(result) = consumer.next().await {
                        match result {
                            Ok(delivery) => match consume(
                                &global::tracer("amqp consumer"),
                                &queue,
                                &msgs_allowed,
                                &handlers,
                                &delivery,
                                m_amqp.clone(),
                            )
                            .await
                            {
                                Err(e) => error!(error = e.to_string(), "errors consume msg"),
                                _ => {}
                            },
                            Err(e) => error!(error = e.to_string(), "error receiving delivery msg"),
                        };
                    }
                }
            }));
        }

        join_all(spawns).await
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use opentelemetry::Context;

    use super::*;
    use crate::mocks::MockAmqpImpl;

    #[test]
    fn test_dispatch_declare_successfully() {
        let mut dispatcher = Dispatcher::new(Arc::new(MockAmqpImpl::new()));
        let handler = MockedHandler { mock_error: None };

        let res = dispatcher.declare(
            QueueDefinition::name("queue"),
            "msg_type".to_owned(),
            Arc::new(handler),
        );

        assert!(res.is_ok());
        assert_eq!(dispatcher.handlers.len(), 1);
        assert_eq!(dispatcher.msgs_types.len(), 1);
        assert_eq!(dispatcher.queues.len(), 1);
    }

    #[test]
    fn test_dispatch_declare_error() {
        let mut dispatcher = Dispatcher::new(Arc::new(MockAmqpImpl::new()));
        let handler = MockedHandler { mock_error: None };

        let res = dispatcher.declare(
            QueueDefinition::name("queue"),
            "".to_owned(),
            Arc::new(handler),
        );

        assert!(res.is_err());
        assert_eq!(dispatcher.handlers.len(), 0);
        assert_eq!(dispatcher.msgs_types.len(), 0);
        assert_eq!(dispatcher.queues.len(), 0);
    }

    struct MockedHandler {
        pub mock_error: Option<AmqpError>,
    }

    #[async_trait]
    impl ConsumerHandler for MockedHandler {
        async fn exec(&self, _ctx: &Context, _data: &[u8]) -> Result<(), AmqpError> {
            if self.mock_error.is_none() {
                return Ok(());
            }

            Err(AmqpError::InternalError {})
        }
    }
}
