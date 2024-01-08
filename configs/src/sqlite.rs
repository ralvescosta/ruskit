#[derive(Debug, Clone)]
pub struct SqliteConfigs {
    ///Default: local.db
    pub file: String,
    ///Default: postgres
    pub user: String,
    /// Default: postgres
    pub password: String,
}

impl Default for SqliteConfigs {
    fn default() -> Self {
        Self {
            file: Default::default(),
            user: Default::default(),
            password: Default::default(),
        }
    }
}
