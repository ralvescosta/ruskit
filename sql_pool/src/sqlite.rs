use errors::sql_pool::SqlPoolError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tracing::error;

pub fn conn_pool(cfg: &env::Config) -> Result<Pool<SqliteConnectionManager>, SqlPoolError> {
    let manager = SqliteConnectionManager::file(&cfg.sqlite.file);

    let pool = r2d2::Pool::new(manager).map_err(|e| {
        error!(error = e.to_string(), "bad sqlite connection");
        SqlPoolError::SqliteConnectionErr(e.to_string())
    })?;

    Ok(pool)
}
