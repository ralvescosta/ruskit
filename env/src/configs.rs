use crate::Environment;

#[derive(Debug, Clone, Default)]
pub struct Configs<T: DynamicConfig> {
    pub app: AppConfig,
    pub mqtt: MQTTConfig,
    pub amqp: AmqpConfig,
    pub otlp: OTLPConfig,
    pub postgres: PostgresConfig,
    pub sqlite: SqliteConfig,
    pub aws: AwsConfig,
    pub dynamo: DynamoConfig,
    pub health_readiness: HealthReadinessConfig,

    pub dynamic: T,

    ///Default: 15000
    pub multiple_message_timer: i32,
}

pub trait DynamicConfig: Default {
    fn load(&self);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty;
impl DynamicConfig for Empty {
    fn load(&self) {}
}

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    ///Default: APP_NAME
    pub name: String,
    ///Default: Environment::Local
    pub env: Environment,
    ///Default:false
    pub use_secret_manager: bool,
    ///Default: context
    pub secret_key: String,
    ///Default: 0.0.0.0
    pub host: String,
    ///Default: 31033
    pub port: u64,
    ///Default: debug
    pub log_level: String,
    ///Default: false
    pub enable_external_creates_logging: bool,
    ///Default:
    pub auth_authority: String,
}

impl AppConfig {
    pub fn app_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MQTTConfig {
    ///Default: localhost
    pub host: String,
    ///Default: 1883
    pub port: u64,
    ///Default: mqtt_user
    pub user: String,
    /// Default: password
    pub password: String,
    ///Used with Public Cloud Brokers
    pub device_name: String,
    ///Used with Public Cloud Brokers
    pub root_ca_path: String,
    ///Used with Public Cloud Brokers
    pub cert_path: String,
    ///Used with Public Cloud Brokers
    pub private_key_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct AmqpConfig {
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

#[derive(Debug, Clone, Default)]
pub struct OTLPConfig {
    ///Default: false
    pub enable_traces: bool,
    ///Default: false
    pub enable_metrics: bool,
    ///Default: localhost
    pub host: String,
    ///Default: key
    pub key: String,
    pub service_type: String,
    ///Default: 30s
    pub export_timeout: u64,
    ///Default: 60s
    pub metrics_export_interval: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PostgresConfig {
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

#[derive(Debug, Clone, Default)]
pub struct SqliteConfig {
    ///Default: local.db
    pub file: String,
    ///Default: postgres
    pub user: String,
    /// Default: postgres
    pub password: String,
}

#[derive(Debug, Clone, Default)]
pub struct AwsConfig {
    ///Default: local
    pub access_key_id: Option<String>,
    ///Default: local
    pub secret_access_key: Option<String>,
    ///Default:
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DynamoConfig {
    ///Default: localhost
    pub endpoint: String,
    ///Default: us-east-1
    pub region: String,
    ///Default: table
    pub table: String,
}

#[derive(Debug, Clone, Default)]
pub struct HealthReadinessConfig {
    ///Default: 8888
    pub port: u64,
    ///Default: false
    pub enable: bool,
}

impl HealthReadinessConfig {
    pub fn health_readiness_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

impl<T> Configs<T>
where
    T: DynamicConfig,
{
    pub fn amqp_uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}{}",
            self.amqp.user, self.amqp.password, self.amqp.host, self.amqp.port, self.amqp.vhost
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_app_addr() {
        let cfg = AppConfig::default();

        assert_eq!(cfg.app_addr(), format!("{}:{}", cfg.host, cfg.port))
    }

    #[test]
    fn should_return_amqp_uri() {
        let cfg = Configs::<Empty>::default();

        assert_eq!(
            cfg.amqp_uri(),
            format!(
                "amqp://{}:{}@{}:{}{}",
                cfg.amqp.user, cfg.amqp.password, cfg.amqp.host, cfg.amqp.port, cfg.amqp.vhost
            )
        )
    }
}
