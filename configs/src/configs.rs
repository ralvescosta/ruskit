use std::fmt::Display;

use crate::Environment;

#[derive(Debug, Clone, Default)]
pub struct Configs<T: DynamicConfigs> {
    pub app: AppConfigs,
    pub auth0: Auth0Configs,
    pub mqtt: MQTTConfigs,
    pub rabbitmq: RabbitMQConfigs,
    pub metric: MetricConfigs,
    pub trace: TraceConfigs,
    pub postgres: PostgresConfigs,
    pub sqlite: SqliteConfigs,
    pub aws: AwsConfigs,
    pub dynamo: DynamoConfigs,
    pub health_readiness: HealthReadinessConfigs,

    pub dynamic: T,

    ///Default: 15000
    pub multiple_message_timer: i32,
}

pub trait DynamicConfigs: Default {
    fn load(&mut self);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty;
impl DynamicConfigs for Empty {
    fn load(&mut self) {}
}

#[derive(Debug, Clone, Default)]
pub enum SecretsManagerKind {
    #[default]
    None,
    AWSSecretManager,
}

impl From<&str> for SecretsManagerKind {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "AWS" => SecretsManagerKind::AWSSecretManager,
            _ => SecretsManagerKind::None,
        }
    }
}

impl From<&String> for SecretsManagerKind {
    fn from(value: &String) -> Self {
        match value.to_uppercase().as_str() {
            "AWS" => SecretsManagerKind::AWSSecretManager,
            _ => SecretsManagerKind::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfigs {
    ///Default: APP_NAME
    pub name: String,
    ///Default: Environment::Local
    pub env: Environment,
    ///Default:false
    pub secret_manager: SecretsManagerKind,
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

impl Default for AppConfigs {
    fn default() -> Self {
        Self {
            name: "APP_NAME".to_owned(),
            env: Environment::Local,
            secret_manager: SecretsManagerKind::default(),
            secret_key: "context".to_owned(),
            host: "0.0.0.0".to_owned(),
            port: 31033,
            log_level: "debug".to_owned(),
            enable_external_creates_logging: false,
        }
    }
}

impl AppConfigs {
    pub fn app_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone)]
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

impl Default for Auth0Configs {
    fn default() -> Self {
        Self {
            domain: Default::default(),
            audience: Default::default(),
            issuer: Default::default(),
            client_id: Default::default(),
            client_secret: Default::default(),
            grant_type: "client_credentials".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum MQTTBrokerKind {
    #[default]
    Default,
    AWSIoTCore,
}

impl From<&str> for MQTTBrokerKind {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "AWSIoTCore" => MQTTBrokerKind::AWSIoTCore,
            _ => MQTTBrokerKind::Default,
        }
    }
}

impl From<&String> for MQTTBrokerKind {
    fn from(value: &String) -> Self {
        match value.to_uppercase().as_str() {
            "AWSIoTCore" => MQTTBrokerKind::AWSIoTCore,
            _ => MQTTBrokerKind::Default,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum MQTTTransport {
    #[default]
    TCP,
    SSL,
    WS,
}

impl From<&str> for MQTTTransport {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "SSL" => MQTTTransport::SSL,
            "WS" => MQTTTransport::WS,
            _ => MQTTTransport::TCP,
        }
    }
}

impl From<&String> for MQTTTransport {
    fn from(value: &String) -> Self {
        match value.to_uppercase().as_str() {
            "SSL" => MQTTTransport::SSL,
            "WS" => MQTTTransport::WS,
            _ => MQTTTransport::TCP,
        }
    }
}

impl Display for MQTTTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MQTTTransport::TCP => write!(f, "tcp"),
            MQTTTransport::SSL => write!(f, "ssl"),
            MQTTTransport::WS => write!(f, "ws"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MQTTConfigs {
    pub broker_kind: MQTTBrokerKind,
    ///Default: localhost
    pub host: String,
    //Default: tcp
    pub transport: MQTTTransport,
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

impl Default for MQTTConfigs {
    fn default() -> Self {
        Self {
            broker_kind: MQTTBrokerKind::default(),
            host: "localhost".to_owned(),
            transport: MQTTTransport::default(),
            port: 1883,
            user: "mqtt".to_owned(),
            password: "password".to_owned(),
            device_name: Default::default(),
            root_ca_path: Default::default(),
            cert_path: Default::default(),
            private_key_path: Default::default(),
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct MetricConfigs {
    ///Default: false
    pub enable: bool,
    ///Only used with OTLP
    ///
    ///Default: localhost
    pub host: String,
    ///Only used with OTLP
    ///
    ///Default: key
    pub key: String,
    pub service_type: String,
    ///Only used with OTLP
    ///
    ///Default: 30s
    pub export_timeout: u64,
    ///Only used with OTLP
    ///
    ///Default: 60s
    pub export_interval: u64,
    ///Only used with OTLP
    ///
    ///Default: 0.8
    pub export_rate_base: f64,
}

impl Default for MetricConfigs {
    fn default() -> Self {
        Self {
            enable: false,
            host: Default::default(),
            key: Default::default(),
            service_type: Default::default(),
            export_timeout: 30,
            export_interval: 60,
            export_rate_base: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceConfigs {
    ///Default: false
    pub enable: bool,
    ///Default: localhost
    pub host: String,
    ///Default: key
    pub key: String,
    pub service_type: String,
    ///Default: 30s
    pub export_timeout: u64,
    ///Default: 60s
    pub export_interval: u64,
    ///Default: 0.8
    pub export_rate_base: f64,
}

impl Default for TraceConfigs {
    fn default() -> Self {
        Self {
            enable: false,
            host: Default::default(),
            key: Default::default(),
            service_type: Default::default(),
            export_timeout: 30,
            export_interval: 60,
            export_rate_base: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
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

impl Default for OTLPConfigs {
    fn default() -> Self {
        Self {
            enable_traces: false,
            enable_metrics: false,
            host: Default::default(),
            key: Default::default(),
            service_type: Default::default(),
            export_timeout: 30,
            metrics_export_interval: 60,
            export_rate_base: 0.8,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct AwsConfigs {
    ///Default: local
    pub access_key_id: Option<String>,
    ///Default: local
    pub secret_access_key: Option<String>,
    ///Default:
    pub session_token: Option<String>,
}

impl Default for AwsConfigs {
    fn default() -> Self {
        Self {
            access_key_id: Some("local".to_owned()),
            secret_access_key: Some("local".to_owned()),
            session_token: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamoConfigs {
    ///Default: localhost
    pub endpoint: String,
    ///Default: us-east-1
    pub region: String,
    ///Default: table
    pub table: String,
    ///Default: 31536000
    pub expire: u64,
}

impl Default for DynamoConfigs {
    fn default() -> Self {
        Self {
            endpoint: "localhost".to_owned(),
            region: "us-east-1".to_owned(),
            table: Default::default(),
            expire: 31536000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthReadinessConfigs {
    ///Default: 8888
    pub port: u64,
    ///Default: false
    pub enable: bool,
}

impl Default for HealthReadinessConfigs {
    fn default() -> Self {
        Self {
            port: 8888,
            enable: false,
        }
    }
}

impl HealthReadinessConfigs {
    pub fn health_readiness_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfigs {
    ///Default: 3600s
    pub jwk_rotate_period: u64,
}

impl Default for AuthConfigs {
    fn default() -> Self {
        Self {
            jwk_rotate_period: 3600,
        }
    }
}

impl<T> Configs<T>
where
    T: DynamicConfigs,
{
    pub fn rabbitmq_uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}{}",
            self.rabbitmq.user,
            self.rabbitmq.password,
            self.rabbitmq.host,
            self.rabbitmq.port,
            self.rabbitmq.vhost
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
            cfg.rabbitmq_uri(),
            format!(
                "amqp://{}:{}@{}:{}{}",
                cfg.rabbitmq.user,
                cfg.rabbitmq.password,
                cfg.rabbitmq.host,
                cfg.rabbitmq.port,
                cfg.rabbitmq.vhost
            )
        )
    }
}
