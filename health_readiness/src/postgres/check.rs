use crate::{errors::HealthReadinessError, HealthChecker};
use deadpool_postgres::Pool;
use std::sync::Arc;
use tracing::error;

pub struct PostgresHealthChecker {
    pool: Arc<Pool>,
}

impl PostgresHealthChecker {
    pub fn new(pool: Arc<Pool>) -> Arc<PostgresHealthChecker> {
        Arc::new(PostgresHealthChecker { pool })
    }
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
        let conn = match self.pool.get().await {
            Err(err) => {
                error!(error = err.to_string(), "error to get conn in pool");
                Err(HealthReadinessError::PostgresError)
            }
            Ok(c) => Ok(c),
        }?;

        match conn.query("SELECT 1;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to ping the database");
                Err(HealthReadinessError::PostgresError)
            }
            _ => Ok(()),
        }
    }
}
