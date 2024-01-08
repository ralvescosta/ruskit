#[derive(Debug, Clone)]
pub struct PostgresConfigs {
    ///Default: localhost
    pub host: String,
    ///Default: postgres
    pub user: String,
    /// Default: postgres
    pub password: String,
    ///Default: 5432
    pub port: u16,
    ///Default: postgres
    pub db: String,
}

impl Default for PostgresConfigs {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            user: Default::default(),
            password: Default::default(),
            port: Default::default(),
            db: Default::default(),
        }
    }
}
