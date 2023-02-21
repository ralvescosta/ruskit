use crate::errors::SqlPoolError;
use deadpool_sqlite::{Config, Pool, Runtime};
use env::SqliteConfig;
use tracing::error;

pub fn conn_pool(cfg: &SqliteConfig) -> Result<Pool, SqlPoolError> {
    let cfg = Config::new(cfg.file.clone());

    let pool = match cfg.create_pool(Runtime::Tokio1) {
        Err(e) => {
            error!(error = e.to_string(), "error to create sqlite conn pool");
            Err(SqlPoolError::SqliteConnectionErr(e.to_string()))
        }
        Ok(p) => Ok(p),
    }?;

    Ok(pool)
}
