use crate::Environment;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub app: AppCfg,
    pub mqtt: MqttCfg,
    pub amqp: AmqpCfg,
    pub otlp: OtlpCfg,
    pub postgres: PostgresCfg,
    pub dynamo: DynamoCfg,
    pub health_readiness: HealthReadinessCfg,

    ///Default: 15000
    pub multiple_message_timer: i32,
}

#[derive(Debug, Clone, Default)]
pub struct AppCfg {
    ///Default: APP_NAME
    pub name: String,
    ///Default: context
    pub ctx: String,
    ///Default: Environment::Local
    pub env: Environment,
    ///Default: 0.0.0.0
    pub host: String,
    ///Default: 31033
    pub port: u64,
    ///Default: debug
    pub log_level: String,
    ///Default: false
    pub enable_external_creates_logging: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MqttCfg {
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
pub struct AmqpCfg {
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
pub struct OtlpCfg {
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
pub struct PostgresCfg {
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
pub struct DynamoCfg {
    ///Default: localhost
    pub host: String,
    ///Default: dynamo
    pub user: String,
    ///Default: dynamo
    pub password: String,
}

#[derive(Debug, Clone, Default)]
pub struct HealthReadinessCfg {
    ///Default: 8888
    pub port: u64,
    ///Default: false
    pub enable: bool,
}

impl Config {
    pub fn app_addr(&self) -> String {
        format!("{}:{}", self.app.host, self.app.port)
    }

    pub fn health_readiness_addr(&self) -> String {
        format!("0.0.0.0:{}", self.health_readiness.port)
    }

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
        let cfg = Config::default();

        assert_eq!(cfg.app_addr(), format!("{}:{}", cfg.app.host, cfg.app.port))
    }

    #[test]
    fn should_return_amqp_uri() {
        let cfg = Config::default();

        assert_eq!(
            cfg.amqp_uri(),
            format!(
                "amqp://{}:{}@{}:{}{}",
                cfg.amqp.user, cfg.amqp.password, cfg.amqp.host, cfg.amqp.port, cfg.amqp.vhost
            )
        )
    }
}
