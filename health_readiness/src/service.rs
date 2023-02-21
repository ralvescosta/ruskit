use crate::errors::HealthReadinessError;
use std::sync::Arc;
use tracing::error;

#[async_trait::async_trait]
pub trait HealthChecker {
    fn name(&self) -> String;
    fn description(&self) -> String;
    async fn check(&self) -> Result<(), HealthReadinessError>;
}

#[async_trait::async_trait]
pub trait HealthReadinessService {
    fn register(&mut self, c: Arc<dyn HealthChecker + Send + Sync>);
    async fn validate(&self) -> Result<(), HealthReadinessError>;
}

pub struct HealthReadinessImpl {
    checkers: Vec<Arc<dyn HealthChecker + Send + Sync>>,
}

impl HealthReadinessImpl {
    pub fn new() -> Arc<dyn HealthReadinessService + Send + Sync> {
        return Arc::new(HealthReadinessImpl { checkers: vec![] });
    }

    pub fn from(
        checkers: Vec<Arc<dyn HealthChecker + Send + Sync>>,
    ) -> Arc<dyn HealthReadinessService + Send + Sync> {
        return Arc::new(HealthReadinessImpl { checkers });
    }
}

#[async_trait::async_trait]
impl HealthReadinessService for HealthReadinessImpl {
    fn register(&mut self, c: Arc<dyn HealthChecker + Send + Sync>) {
        self.checkers.push(c);
    }

    async fn validate(&self) -> Result<(), HealthReadinessError> {
        for checker in self.checkers.clone() {
            checker.check().await.map_err(|e| {
                error!(error = e.to_string(), "{:?}", checker.name());
                e
            })?;
        }

        Ok(())
    }
}
