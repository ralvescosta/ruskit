use crate::errors::SqlPoolError;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use env::PostgresConfig;
use tokio_postgres::NoTls;
use tracing::error;

pub fn conn_pool(cfg: &PostgresConfig) -> Result<Pool, SqlPoolError> {
    let mut pg_cfg = tokio_postgres::Config::new();
    pg_cfg.host(&cfg.host);
    pg_cfg.port(cfg.port);
    pg_cfg.dbname(&cfg.db);
    pg_cfg.user(&cfg.user);
    pg_cfg.password(&cfg.password);

    let mgr_cfg = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = Manager::from_config(pg_cfg, NoTls, mgr_cfg);

    let pool = match Pool::builder(mgr).max_size(16).build() {
        Err(e) => {
            error!(error = e.to_string(), "error to create postgres conn pool");
            Err(SqlPoolError::PostgresConnectionErr(e.to_string()))
        }
        Ok(p) => Ok(p),
    }?;

    Ok(pool)
}
