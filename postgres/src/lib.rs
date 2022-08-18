use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use env;
use std::error::Error;
use tokio_postgres::NoTls;

pub fn conn_pool(cfg: &env::Config) -> Result<Pool, Box<dyn Error>> {
    let mut pg_cfg = tokio_postgres::Config::new();
    pg_cfg.host(&cfg.db_host);
    pg_cfg.port(cfg.db_port);
    pg_cfg.dbname(&cfg.db_name);
    pg_cfg.user(&cfg.db_user);
    pg_cfg.password(&cfg.db_password);

    let mgr_cfg = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = Manager::from_config(pg_cfg, NoTls, mgr_cfg);

    let pool = Pool::builder(mgr).max_size(16).build()?;

    Ok(pool)
}
