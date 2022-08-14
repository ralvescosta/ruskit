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
    pub app_name: &'static str,
    pub env: Environment,
    pub app_host: &'static str,
    pub app_port: u64,
    pub log_level: &'static str,
    pub enable_external_creates_logging: bool,

    pub mqtt_host: &'static str,
    pub mqtt_port: u16,
    pub mqtt_user: &'static str,
    pub mqtt_password: &'static str,

    pub amqp_host: &'static str,
    pub amqp_port: u16,
    pub amqp_user: &'static str,
    pub amqp_password: &'static str,
    pub amqp_vhost: &'static str,

    pub otlp_host: &'static str,
    pub otlp_key: &'static str,
    pub otlp_service_type: &'static str,
    pub otlp_export_time: u64,

    pub db_host: &'static str,
    pub db_user: &'static str,
    pub db_password: &'static str,
    pub db_port: u16,
    pub db_name: &'static str,
}

impl Config {
    pub fn new() -> Box<Self> {
        Box::new(Config {
            app_name: "rust_iot",
            app_host: "local",
            app_port: 12345,
            env: Environment::Local,
            log_level: "debug",
            enable_external_creates_logging: false,

            mqtt_host: "localhost",
            mqtt_port: 1883,
            mqtt_user: "mqtt_user",
            mqtt_password: "password",

            amqp_host: "localhost",
            amqp_port: 5672,
            amqp_user: "admin",
            amqp_password: "password",
            amqp_vhost: "",

            otlp_host: "https://otlp.nr-data.net:4317",
            otlp_key: "",
            otlp_service_type: "MQTT",
            otlp_export_time: 10,

            db_host: "locahost",
            db_user: "postgres",
            db_password: "postgres",
            db_port: 5432,
            db_name: "postgres",
        })
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

    pub fn mock() -> Box<Self> {
        Box::new(Config {
            app_name: "rust_iot",
            app_host: "local",
            app_port: 12345,
            env: Environment::Local,
            mqtt_host: "localhost",
            mqtt_port: 1883,
            mqtt_user: "mqtt_user",
            mqtt_password: "password",
            log_level: "debug",
            enable_external_creates_logging: false,
            amqp_host: "amqp://localhost",
            amqp_port: 5672,
            amqp_user: "admin",
            amqp_password: "password",
            amqp_vhost: "",
            otlp_host: "https://otlp.nr-data.net:4317",
            otlp_key: "some_key",
            otlp_service_type: "MQTT",
            otlp_export_time: 10,
            db_host: "locahost",
            db_user: "postgres",
            db_password: "password",
            db_port: 5432,
            db_name: "test",
        })
    }
}
