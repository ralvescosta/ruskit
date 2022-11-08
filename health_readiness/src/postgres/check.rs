use crate::HealthChecker;
use deadpool_postgres::Pool;
use errors::health_readiness::HealthReadinessError;
use std::sync::Arc;
use tracing::error;

pub struct PostgresHealthChecker {
    pool: Arc<Pool>,
}

#[async_trait::async_trait]
impl HealthChecker for PostgresHealthChecker {
    fn name(&self) -> String {
        "Postgres health readiness".to_owned()
    }

    fn description(&self) -> String {
        "Postgres health readiness".to_owned()
    }

    async fn check(&self) -> Result<(), HealthReadinessError> {
        let conn = self.pool.get().await.map_err(|e| {
            error!(error = e.to_string(), "error to get conn in pool");
            HealthReadinessError::PostgresError
        })?;

        conn.query("SELECT 1;", &[]).await.map_err(|e| {
            error!(error = e.to_string(), "error to ping the database");
            HealthReadinessError::PostgresError
        })?;

        Ok(())
    }
}
