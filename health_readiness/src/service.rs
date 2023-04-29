use crate::{
    errors::HealthReadinessError, mqtt::MqttHealthChecker, postgres::PostgresHealthChecker,
    rabbitmq::RabbitMqHealthChecker,
};
use async_trait::async_trait;
use deadpool_postgres::Pool;
use lapin::Connection;
use paho_mqtt::AsyncClient;
use std::sync::Arc;
use tracing::error;
#[async_trait]
pub trait HealthChecker: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    async fn check(&self) -> Result<(), HealthReadinessError>;
}

#[async_trait]
pub trait HealthReadinessService: Send + Sync {
    fn register(&mut self, c: Arc<dyn HealthChecker>);
    async fn validate(&self) -> Result<(), HealthReadinessError>;
}

#[derive(Default)]
pub struct HealthReadinessServiceImpl {
    checkers: Vec<Arc<dyn HealthChecker>>,
}

impl HealthReadinessServiceImpl {
    pub fn new(checkers: Vec<Arc<dyn HealthChecker>>) -> Arc<HealthReadinessServiceImpl> {
        return Arc::new(HealthReadinessServiceImpl { checkers });
    }

    pub fn mqtt(mut self, client: Arc<AsyncClient>) -> Self {
        self.checkers.push(MqttHealthChecker::new(client));
        self
    }

    pub fn amqp(mut self, conn: Arc<Connection>) -> Self {
        self.checkers.push(RabbitMqHealthChecker::new(conn));
        self
    }

    pub fn postgres(mut self, pool: Arc<Pool>) -> Self {
        self.checkers.push(PostgresHealthChecker::new(pool));
        self
    }
}

#[async_trait]
impl HealthReadinessService for HealthReadinessServiceImpl {
    fn register(&mut self, c: Arc<dyn HealthChecker>) {
        self.checkers.push(c);
    }

    async fn validate(&self) -> Result<(), HealthReadinessError> {
        for checker in self.checkers.clone() {
            match checker.check().await {
                Err(err) => {
                    error!(error = err.to_string(), "{:?}", checker.name());
                    Err(err)
                }
                _ => Ok(()),
            }?;
        }

        Ok(())
    }
}
