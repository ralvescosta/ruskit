use crate::HealthChecker;
use errors::health_readiness::HealthReadinessError;
use lapin::Connection;
use std::sync::Arc;

pub struct RabbitMqHealthChecker {
    conn: Arc<Connection>,
}

impl RabbitMqHealthChecker {
    pub fn new(conn: Arc<Connection>) -> Arc<dyn HealthChecker + Send + Sync> {
        Arc::new(RabbitMqHealthChecker { conn })
    }
}

#[async_trait::async_trait]
impl HealthChecker for RabbitMqHealthChecker {
    fn name(&self) -> String {
        "RabbitMq health readiness".to_owned()
    }

    fn description(&self) -> String {
        "RabbitMq health readiness".to_owned()
    }

    async fn check(&self) -> Result<(), HealthReadinessError> {
        let status = self.conn.status();
        if status.connected() {
            return Ok(());
        }

        Err(HealthReadinessError::RabbitMqError)
    }
}
