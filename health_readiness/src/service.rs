use crate::errors::HealthReadinessError;
#[cfg(feature = "mqtt")]
use crate::mqtt::MqttHealthChecker;
#[cfg(feature = "postgres")]
use crate::postgres::PostgresHealthChecker;
#[cfg(feature = "rabbitmq")]
use crate::rabbitmq::RabbitMqHealthChecker;
use async_trait::async_trait;
#[cfg(feature = "postgres")]
use deadpool_postgres::Pool;
#[cfg(feature = "rabbitmq")]
use lapin::Connection;
#[cfg(feature = "mqtt")]
use paho_mqtt::AsyncClient;
use std::{sync::Arc, vec};
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
    pub fn empty() -> Arc<HealthReadinessServiceImpl> {
        return Arc::new(HealthReadinessServiceImpl { checkers: vec![] });
    }

    pub fn new(checkers: Vec<Arc<dyn HealthChecker>>) -> Arc<HealthReadinessServiceImpl> {
        return Arc::new(HealthReadinessServiceImpl { checkers });
    }

    #[cfg(feature = "mqtt")]
    pub fn mqtt(mut self, client: Arc<AsyncClient>) -> Self {
        self.checkers.push(MqttHealthChecker::new(client));
        self
    }

    #[cfg(feature = "rabbitmq")]
    pub fn rabbitmq(mut self, conn: Arc<Connection>) -> Self {
        self.checkers.push(RabbitMqHealthChecker::new(conn));
        self
    }

    #[cfg(feature = "postgres")]
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
