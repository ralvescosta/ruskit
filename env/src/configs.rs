use dotenv::from_filename;
use std::{collections::HashMap, env};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Environment {
    #[default]
    Local,
    Dev,
    Staging,
    Prod,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub app_name: String,
    pub env: Environment,
    pub app_host: String,
    pub app_port: u64,
    pub log_level: String,
    pub enable_external_creates_logging: bool,

    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_user: String,
    pub mqtt_password: String,

    pub amqp_host: String,
    pub amqp_port: u16,
    pub amqp_user: String,
    pub amqp_password: String,
    pub amqp_vhost: String,

    pub otlp_host: String,
    pub otlp_key: String,
    pub otlp_service_type: String,
    pub otlp_export_time: u64,

    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
    pub db_port: u16,
    pub db_name: String,
}

impl Config {
    pub fn new() -> Config {
        let e = Config::load_file();

        let mut map = HashMap::new();
        Config::load_from_env(&mut map);

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
                .unwrap_or(&String::from("12345"))
                .to_string()
                .parse()
                .unwrap_or(12345),
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

            mqtt_password: map
                .get("MQTT_PASSWORD")
                .unwrap_or(&String::from("password"))
                .to_string(),

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

            amqp_password: map
                .get("AMQP_PASSWORD")
                .unwrap_or(&String::from("guest"))
                .to_string(),

            amqp_vhost: map
                .get("AMQP_VHOST")
                .unwrap_or(&String::from(""))
                .to_string(),

            //
            otlp_host: map
                .get("OTLP_HOST")
                .unwrap_or(&String::from("https://otlp.nr-data.net:4317"))
                .to_string(),

            otlp_key: map
                .get("OTLP_KEY")
                .unwrap_or(&String::from("e84b3e41a69635447392533e627aac0c56c5NRAL"))
                .to_string(),

            otlp_service_type: map
                .get("OTLP_SERVICE_TYPE")
                .unwrap_or(&String::from("MQTT"))
                .to_string(),

            otlp_export_time: map
                .get("OTLP_EXPORT_TIME")
                .unwrap_or(&String::from("10"))
                .to_string()
                .parse()
                .unwrap_or(10),

            //
            db_host: map
                .get("DB_HOST")
                .unwrap_or(&String::from("localhost"))
                .to_string(),

            db_user: map
                .get("DB_USER")
                .unwrap_or(&String::from("postgres"))
                .to_string(),

            db_password: map
                .get("DB_PASSWORD")
                .unwrap_or(&String::from("postgres"))
                .to_string(),

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

    fn load_file() -> Environment {
        let env = env::var("RUST_ENV");
        if env.is_err() {
            from_filename(".env.local").ok();
            return Environment::Local;
        }

        match env.unwrap().as_str() {
            "production" | "prod" | "PRODUCTION" | "PROD" => {
                from_filename(".env.production").ok();
                Environment::Prod
            }
            "staging" | "stg" | "STAGING" | "STG" => {
                from_filename(".env.staging").ok();
                Environment::Staging
            }
            "develop" | "DEVELOP" | "dev" | "DEV" => {
                from_filename(".env.develop").ok();
                Environment::Dev
            }
            _ => {
                from_filename(".env.local").ok();
                Environment::Local
            }
        }
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
            otlp_host: "https://otlp.nr-data.net:4317".to_owned(),
            otlp_key: "some_key".to_owned(),
            otlp_service_type: "MQTT".to_owned(),
            otlp_export_time: 10,
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
