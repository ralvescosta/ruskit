use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use env;
use errors::sql_pool::SqlPoolError;
use tokio_postgres::NoTls;
use tracing::error;

pub fn conn_pool(cfg: &env::Config) -> Result<Pool, SqlPoolError> {
    let mut pg_cfg = tokio_postgres::Config::new();
    pg_cfg.host(&cfg.postgres.host);
    pg_cfg.port(cfg.postgres.port);
    pg_cfg.dbname(&cfg.postgres.db);
    pg_cfg.user(&cfg.postgres.user);
    pg_cfg.password(&cfg.postgres.password);

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
