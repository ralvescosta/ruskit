use deadpool_sqlite::{Config, Pool, Runtime};
use errors::sql_pool::SqlPoolError;
use tracing::error;

pub fn conn_pool(cfg: &env::Config) -> Result<Pool, SqlPoolError> {
    let cfg = Config::new(cfg.sqlite.file.clone());

    let pool = match cfg.create_pool(Runtime::Tokio1) {
        Err(e) => {
            error!(error = e.to_string(), "error to create sqlite conn pool");
            Err(SqlPoolError::SqliteConnectionErr(e.to_string()))
        }
        Ok(p) => Ok(p),
    }?;

    Ok(pool)
}
