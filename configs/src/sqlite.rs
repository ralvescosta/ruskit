#[derive(Debug, Clone, Default)]
pub struct SqliteConfigs {
    ///Default: local.db
    pub file: String,
    ///Default: postgres
    pub user: String,
    /// Default: postgres
    pub password: String,
}
