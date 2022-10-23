use dotenv::from_filename;
use std::{collections::HashMap, env, fmt};
use tracing::error;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Environment {
    #[default]
    Local,
    Dev,
    Staging,
    Prod,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Environment::Local => "local",
            Environment::Dev => "development",
            Environment::Staging => "staging",
            Environment::Prod => "prod",
        };
        write!(f, "{}", printable)
    }
}

impl Environment {
    pub fn is_local(&self) -> bool {
        self == &Environment::Local
    }

    pub fn is_dev(&self) -> bool {
        self == &Environment::Dev
    }

    pub fn is_stg(&self) -> bool {
        self == &Environment::Staging
    }

    pub fn is_prod(&self) -> bool {
        self == &Environment::Prod
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    ///Default: APP_NAME
    pub app_name: String,
    ///Default: Environment::Local
    pub env: Environment,
    ///Default: 0.0.0.0
    pub app_host: String,
    ///Default: 31033
    pub app_port: u64,
    ///Default: debug
    pub log_level: String,
    ///Default: false
    pub enable_external_creates_logging: bool,

    ///Default: localhost
    pub mqtt_host: String,
    ///Default: 1883
    pub mqtt_port: u16,
    ///Default: mqtt_user
    pub mqtt_user: String,
    /// When the password will be read, we will apply a Base64 decoded so the password needed to be encoded by Base64
    ///
    /// Default: password
    pub mqtt_password: String,

    ///Default: localhost
    pub amqp_host: String,
    ///Default: 5672
    pub amqp_port: u16,
    ///Default: guest
    pub amqp_user: String,
    /// When the password will be read, we will apply a Base64 decoded so the password needed to be encoded by Base64
    ///
    /// Default: guest
    pub amqp_password: String,
    pub amqp_vhost: String,

    ///Default: false
    pub enable_traces: bool,
    ///Default: false
    pub enable_metrics: bool,
    ///Default: https://otlp.nr-data.net:4317
    pub otlp_host: String,
    ///Default: d1a26ee8b5c8f32c8ecf69c6038b72bd6d79NRAL
    pub otlp_key: String,
    pub otlp_service_type: String,
    ///Default: 30s
    pub otlp_export_timeout: u64,
    ///Default: 60s
    pub otlp_metrics_export_interval: u64,

    ///Default: localhost
    pub db_host: String,
    ///Default: postgres
    pub db_user: String,
    /// When the password will be read, we will apply a Base64 decoded so the password needed to be encoded by Base64
    ///
    /// Default: postgres
    pub db_password: String,
    ///Default: 5432
    pub db_port: u16,
    pub db_name: String,
}

impl Config {
    pub fn new() -> Config {
        let e = Self::load_from_file();

        let mut map = HashMap::new();
        Self::load_from_env(&mut map);

        Config {
            app_name: map
                .get("APP_NAME")
                .unwrap_or(&String::from("APP_NAME"))
                .to_string(),

            app_host: map
                .get("HOST_NAME")
                .unwrap_or(&String::from("HOST_NAME"))
                .to_string(),

            app_port: map
                .get("APP_PORT")
                .unwrap_or(&String::from("31033"))
                .to_string()
                .parse()
                .unwrap_or(31033),
            env: e,

            log_level: map
                .get("LOG_LEVEL")
                .unwrap_or(&String::from("debug"))
                .to_string(),

            enable_external_creates_logging: false,

            //
            mqtt_host: map
                .get("MQTT_HOST")
                .unwrap_or(&String::from("localhost"))
                .to_string(),

            mqtt_port: map
                .get("MQTT_PORT")
                .unwrap_or(&String::from("1883"))
                .to_string()
                .parse()
                .unwrap_or(1883),

            mqtt_user: map
                .get("MQTT_USER")
                .unwrap_or(&String::from("mqtt_user"))
                .to_string(),

            mqtt_password: Self::decoded(
                map.get("MQTT_PASSWORD")
                    .unwrap_or(&String::from("cGFzc3dvcmQ="))
                    .to_string(),
            )
            .unwrap_or_default(),

            //
            amqp_host: map
                .get("AMQP_HOST")
                .unwrap_or(&String::from("localhost"))
                .to_string(),

            amqp_port: map
                .get("AMQP_PORT")
                .unwrap_or(&String::from("5672"))
                .to_string()
                .parse()
                .unwrap_or(5672),

            amqp_user: map
                .get("AMQP_USER")
                .unwrap_or(&String::from("guest"))
                .to_string(),

            amqp_password: Self::decoded(
                map.get("AMQP_PASSWORD")
                    .unwrap_or(&String::from("Z3Vlc3Q="))
                    .to_string(),
            )
            .unwrap_or_default(),

            amqp_vhost: map
                .get("AMQP_VHOST")
                .unwrap_or(&String::from(""))
                .to_string(),

            //
            enable_traces: map
                .get("ENABLE_TRACES")
                .unwrap_or(&String::from("false"))
                .parse()
                .unwrap_or(false),

            enable_metrics: map
                .get("ENABLE_METRICS")
                .unwrap_or(&String::from("false"))
                .parse()
                .unwrap_or(false),

            otlp_host: map
                .get("OTLP_HOST")
                .unwrap_or(&String::from("https://otlp.nr-data.net:4317"))
                .to_string(),

            otlp_key: map
                .get("OTLP_KEY")
                .unwrap_or(&String::from("d1a26ee8b5c8f32c8ecf69c6038b72bd6d79NRAL"))
                .to_string(),

            otlp_service_type: map
                .get("OTLP_SERVICE_TYPE")
                .unwrap_or(&String::from("MQTT"))
                .to_string(),

            otlp_export_timeout: map
                .get("OTLP_EXPORT_TIMEOUT")
                .unwrap_or(&String::from("30"))
                .to_string()
                .parse()
                .unwrap_or(30),

            otlp_metrics_export_interval: map
                .get("OTLP_EXPORT_TIMEOUT")
                .unwrap_or(&String::from("60"))
                .to_string()
                .parse()
                .unwrap_or(60),

            //
            db_host: map
                .get("DB_HOST")
                .unwrap_or(&String::from("localhost"))
                .to_string(),

            db_user: map
                .get("DB_USER")
                .unwrap_or(&String::from("postgres"))
                .to_string(),

            db_password: Self::decoded(
                map.get("DB_PASSWORD")
                    .unwrap_or(&String::from("cG9zdGdyZXM="))
                    .to_string(),
            )
            .unwrap_or_default(),

            db_port: map
                .get("DB_PORT")
                .unwrap_or(&String::from("5432"))
                .to_string()
                .parse()
                .unwrap_or(5432),

            db_name: map
                .get("DB_NAME")
                .unwrap_or(&String::from("postgres"))
                .to_string(),
        }
    }

    fn load_from_file() -> Environment {
        let env = env::var("RUST_ENV");

        if env.is_err() {
            return Self::local();
        }

        match env.unwrap().as_str() {
            "production" | "prod" | "PRODUCTION" | "PROD" => {
                from_filename("./.env.production").ok();
                Environment::Prod
            }
            "staging" | "stg" | "STAGING" | "STG" => {
                from_filename("./.env.staging").ok();
                Environment::Staging
            }
            "develop" | "DEVELOP" | "dev" | "DEV" => {
                from_filename("./.env.develop").ok();
                Environment::Dev
            }
            _ => Self::local(),
        }
    }

    fn local() -> Environment {
        from_filename("./.env.local").ok();
        Environment::Local
    }

    fn load_from_env(map: &mut HashMap<String, String>) {
        for (key, value) in env::vars() {
            map.insert(key, value);
        }
    }

    fn _load_from_secret_manager(map: &mut HashMap<String, String>) {
        for (k, v) in map.clone() {
            if v.is_empty() {
                // get secret from aws
                map.insert(k.to_string(), String::from("secret"));
            }
        }
    }

    pub fn app_addr(&self) -> String {
        format!("{}:{}", self.app_host, self.app_port)
    }

    pub fn amqp_uri(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}{}",
            self.amqp_user, self.amqp_password, self.amqp_host, self.amqp_port, self.amqp_vhost
        )
    }

    pub fn pg_uri(&self) -> String {
        format!(
            "postgresql://{}:{}?dbname={}&user={}&password={}",
            self.db_host, self.db_port, self.db_name, self.db_user, self.db_password
        )
    }

    pub fn decoded(text: String) -> Result<String, ()> {
        let d = base64::decode(text).map_err(|e| {
            error!(error = e.to_string(), "base64 decoded error");
            ()
        })?;

        let string = String::from_utf8(d).map_err(|e| {
            error!(error = e.to_string(), "error to convert to String");
            ()
        })?;

        Ok(string)
    }

    pub fn mock() -> Config {
        Config {
            app_name: "rust_iot".to_owned(),
            app_host: "local".to_owned(),
            app_port: 12345,
            env: Environment::Local,
            mqtt_host: "localhost".to_owned(),
            mqtt_port: 1883,
            mqtt_user: "mqtt_user".to_owned(),
            mqtt_password: "password".to_owned(),
            log_level: "debug".to_owned(),
            enable_external_creates_logging: false,
            amqp_host: "amqp://localhost".to_owned(),
            amqp_port: 5672,
            amqp_user: "admin".to_owned(),
            amqp_password: "password".to_owned(),
            amqp_vhost: "".to_owned(),
            enable_traces: true,
            enable_metrics: true,
            otlp_host: "https://otlp.nr-data.net:4317".to_owned(),
            otlp_key: "some_key".to_owned(),
            otlp_service_type: "MQTT".to_owned(),
            otlp_export_timeout: 10,
            otlp_metrics_export_interval: 60,
            db_host: "locahost".to_owned(),
            db_user: "postgres".to_owned(),
            db_password: "password".to_owned(),
            db_port: 5432,
            db_name: "test".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_config_without_panic() {
        let cfg = Config::new();

        assert_eq!(cfg.app_host, "HOST_NAME")
    }

    #[test]
    fn should_return_app_addr() {
        let cfg = Config::mock();

        assert_eq!(cfg.app_addr(), format!("{}:{}", cfg.app_host, cfg.app_port))
    }

    #[test]
    fn should_return_amqp_uri() {
        let cfg = Config::mock();

        assert_eq!(
            cfg.amqp_uri(),
            format!(
                "amqp://{}:{}@{}:{}{}",
                cfg.amqp_user, cfg.amqp_password, cfg.amqp_host, cfg.amqp_port, cfg.amqp_vhost
            )
        )
    }

    #[test]
    fn should_return_pg_uri() {
        let cfg = Config::mock();

        assert_eq!(
            cfg.pg_uri(),
            format!(
                "postgresql://{}:{}?dbname={}&user={}&password={}",
                cfg.db_host, cfg.db_port, cfg.db_name, cfg.db_user, cfg.db_password
            )
        )
    }
}
