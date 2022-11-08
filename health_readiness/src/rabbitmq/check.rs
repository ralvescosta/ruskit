use crate::HealthChecker;
use errors::health_readiness::HealthReadinessError;
use lapin::Connection;
use std::sync::Arc;

pub struct RabbitMqHealthChecker {
    conn: Arc<Connection>,
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
