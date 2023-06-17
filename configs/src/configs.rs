use crate::Environment;

#[derive(Debug, Clone, Default)]
pub struct Configs<T: DynamicConfigs> {
    pub app: AppConfigs,
    pub auth0: Auth0Configs,
    pub mqtt: MQTTConfigs,
    pub amqp: AmqpConfigs,
    pub otlp: OTLPConfigs,
    pub postgres: PostgresConfigs,
    pub sqlite: SqliteConfigs,
    pub aws: AwsConfigs,
    pub dynamo: DynamoConfigs,
    pub health_readiness: HealthReadinessConfigs,

    pub dynamic: T,
}

pub trait DynamicConfigs: Default {
    fn load(&self);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty;
impl DynamicConfigs for Empty {
    fn load(&self) {}
}

#[derive(Debug, Clone, Default)]
pub struct AppConfigs {
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
}

impl AppConfigs {
    pub fn app_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Auth0Configs {
    //Default: ""
    pub domain: String,
    //Default: ""
    pub audience: String,
    //Default: ""
    pub issuer: String,
    //Default: ""
    pub client_id: String,
    //Default: ""
    pub client_secret: String,
    //Default: "client_credentials"
    pub grant_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct MQTTConfigs {
    ///Default: localhost
    pub host: String,
    //Default: tcp
    pub transport: String,
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
pub struct AmqpConfigs {
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
pub struct OTLPConfigs {
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
    ///Default: 0.8
    pub export_rate_base: f64,
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub struct SqliteConfigs {
    ///Default: local.db
    pub file: String,
    ///Default: postgres
    pub user: String,
    /// Default: postgres
    pub password: String,
}

#[derive(Debug, Clone, Default)]
pub struct AwsConfigs {
    ///Default: local
    pub access_key_id: Option<String>,
    ///Default: local
    pub secret_access_key: Option<String>,
    ///Default:
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DynamoConfigs {
    ///Default: localhost
    pub endpoint: String,
    ///Default: us-east-1
    pub region: String,
    ///Default: table
    pub table: String,
}

#[derive(Debug, Clone, Default)]
pub struct HealthReadinessConfigs {
    ///Default: 8888
    pub port: u64,
    ///Default: false
    pub enable: bool,
}

impl HealthReadinessConfigs {
    pub fn health_readiness_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthConfigs {
    ///Default: 3600s
    pub jwk_rotate_period: u64,
}

impl<T> Configs<T>
where
    T: DynamicConfigs,
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
        let cfg = AppConfigs::default();

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
