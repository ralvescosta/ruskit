use std::sync::Arc;
use tracing::error;

#[async_trait::async_trait]
pub trait HealthChecker {
    fn name(&self) -> String;
    fn description(&self) -> String;
    async fn check(&self) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
pub trait HealthReadiness {
    fn register(&mut self, c: Arc<dyn HealthChecker + Send + Sync>);
    async fn http_handler(&self);
}

pub struct HealthReadinessImpl {
    checkers: Vec<Arc<dyn HealthChecker + Send + Sync>>,
}

#[async_trait::async_trait]
impl HealthReadiness for HealthReadinessImpl {
    fn register(&mut self, c: Arc<dyn HealthChecker + Send + Sync>) {
        self.checkers.push(c);
    }

    async fn http_handler(&self) {
        for checker in self.checkers.clone() {
            checker
                .check()
                .await
                .map_err(|e| error!(error = e.to_string(), "{:?}", checker.name()));
        }
    }
}
