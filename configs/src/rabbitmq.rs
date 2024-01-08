#[derive(Debug, Clone)]
pub struct RabbitMQConfigs {
    ///Default: localhost
    pub host: String,
    ///Default: 5672
    pub port: u64,
    ///Default: guest
    pub user: String,
    /// Default: guest
    pub password: String,
    pub vhost: String,
}

impl Default for RabbitMQConfigs {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            port: 5672,
            user: "default".to_owned(),
            password: "default".to_owned(),
            vhost: Default::default(),
        }
    }
}
