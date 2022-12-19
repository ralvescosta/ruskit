use crate::HealthChecker;
use errors::health_readiness::HealthReadinessError;
use paho_mqtt::AsyncClient;
use std::sync::Arc;
use tracing::debug;

pub struct MqttHealthChecker {
    client: Arc<AsyncClient>,
}

impl MqttHealthChecker {
    pub fn new(client: Arc<AsyncClient>) -> Arc<dyn HealthChecker + Send + Sync> {
        Arc::new(MqttHealthChecker { client })
    }
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
        debug!("mqtt health readiness checking...");
        if self.client.is_connected() {
            return Ok(());
        }
        debug!("checked");
        Err(HealthReadinessError::MqttError)
    }
}
