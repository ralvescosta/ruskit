use crate::errors::SqlPoolError;
use configs::{Configs, DynamicConfigs};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{config::SslMode, tls::NoTls};
use tracing::error;

pub fn conn_pool<T>(cfg: &Configs<T>) -> Result<Pool, SqlPoolError>
where
    T: DynamicConfigs,
{
    let mut pg_cfg = tokio_postgres::Config::new();
    pg_cfg.host(&cfg.postgres.host);
    pg_cfg.port(cfg.postgres.port);
    pg_cfg.dbname(&cfg.postgres.db);
    pg_cfg.user(&cfg.postgres.user);
    pg_cfg.password(&cfg.postgres.password);
    pg_cfg.ssl_mode(SslMode::Disable);
    pg_cfg.application_name(&cfg.app.name);

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
