use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use env;
use std::error::Error;
use tokio_postgres::NoTls;

pub fn conn_pool(cfg: &env::Config) -> Result<Pool, Box<dyn Error>> {
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

    let pool = Pool::builder(mgr).max_size(16).build()?;

    Ok(pool)
}
