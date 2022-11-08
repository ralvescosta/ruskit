use crate::HealthChecker;
use errors::health_readiness::HealthReadinessError;
use paho_mqtt::AsyncClient;
use std::sync::Arc;

pub struct MqttHealthChecker {
    client: Arc<AsyncClient>,
}

#[async_trait::async_trait]
impl HealthChecker for MqttHealthChecker {
    fn name(&self) -> String {
        "Mqtt health readiness".to_owned()
    }

    fn description(&self) -> String {
        "Mqtt health readiness".to_owned()
    }

    async fn check(&self) -> Result<(), HealthReadinessError> {
        if self.client.is_connected() {
            return Ok(());
        }

        Err(HealthReadinessError::MqttError)
    }
}
