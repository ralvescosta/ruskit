use crate::{
    configs::{AppConfig, Configs, DynamicConfig},
    def::{
        AMQP_HOST_ENV_KEY, AMQP_PASSWORD_ENV_KEY, AMQP_PORT_ENV_KEY, AMQP_USER_ENV_KEY,
        AMQP_VHOST_ENV_KEY, APP_NAME_ENV_KEY, APP_PORT_ENV_KEY, AWS_DEFAULT_REGION,
        CUSTOM_AWS_ACCESS_KEY_ID_ENV_KEY, CUSTOM_AWS_SECRET_ACCESS_KEY, DYNAMO_ENDPOINT_ENV_KEY,
        DYNAMO_REGION_ENV_KEY, DYNAMO_TABLE_ENV_KEY, ENABLE_HEALTH_READINESS_ENV_KEY,
        ENABLE_METRICS_ENV_KEY, ENABLE_TRACES_ENV_KEY, HEALTH_READINESS_PORT_ENV_KEY,
        HOST_NAME_ENV_KEY, LOG_LEVEL_ENV_KEY, MQTT_HOST_ENV_KEY, MQTT_PASSWORD_ENV_KEY,
        MQTT_PORT_ENV_KEY, MQTT_USER_ENV_KEY, OTLP_ACCESS_KEY_ENV_KEY, OTLP_EXPORT_TIMEOUT_ENV_KEY,
        OTLP_HOST_ENV_KEY, OTLP_SERVICE_TYPE_ENV_KEY, POSTGRES_DB_ENV_KEY, POSTGRES_HOST_ENV_KEY,
        POSTGRES_PASSWORD_ENV_KEY, POSTGRES_PORT_ENV_KEY, POSTGRES_USER_ENV_KEY,
        SECRET_KEY_ENV_KEY, SECRET_PREFIX, SECRET_PREFIX_TO_DECODE, SQLITE_FILE_NAME_ENV_KEY,
    },
    Environment,
};
use dotenvy::from_filename;
use errors::configs::ConfigsError;
use secrets_manager::{AwsSecretClientBuilder, DummyClient, SecretClient};
use std::{env, sync::Arc};
use tracing::error;

#[derive(Default, Clone, Copy)]
pub enum SecretClientKind {
    #[default]
    None,
    AWSSecreteManager,
}

#[derive(Default)]
pub struct ConfigBuilder {
    secret_client_kind: SecretClientKind,
    client: Option<Arc<dyn SecretClient>>,
    app_cfg: AppConfig,
    mqtt: bool,
    amqp: bool,
    postgres: bool,
    sqlite: bool,
    aws: bool,
    dynamo: bool,
    otlp: bool,
    health: bool,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn mqtt(mut self) -> Self {
        self.mqtt = true;
        self
    }

    pub fn amqp(mut self) -> Self {
        self.amqp = true;
        self
    }

    pub fn postgres(mut self) -> Self {
        self.postgres = true;
        self
    }

    pub fn sqlite(mut self) -> Self {
        self.sqlite = true;
        self
    }

    pub fn dynamodb(mut self) -> Self {
        self.dynamo = true;
        self
    }

    pub fn aws(mut self) -> Self {
        self.aws = true;
        self
    }

    pub fn otlp(mut self) -> Self {
        self.otlp = true;
        self
    }

    pub fn health(mut self) -> Self {
        self.health = true;
        self
    }

    pub fn load_from_aws_secret(mut self) -> Self {
        self.secret_client_kind = SecretClientKind::AWSSecreteManager;
        self
    }

    pub fn load_from_file(mut self) -> Self {
        self.secret_client_kind = SecretClientKind::None;
        self
    }

    pub fn laze_load(mut self) -> (AppConfig, Self) {
        let env = Environment::from_rust_env();

        match env {
            Environment::Prod => {
                from_filename("./.env.prod").ok();
            }
            Environment::Staging => {
                from_filename("./.env.staging").ok();
            }
            Environment::Dev => {
                from_filename("./.env.develop").ok();
            }
            _ => {
                from_filename("./.env.local").ok();
            }
        }

        let name = env::var(APP_NAME_ENV_KEY).unwrap_or_default();
        let secret_key = env::var(SECRET_KEY_ENV_KEY).unwrap_or_default();
        let host = env::var(HOST_NAME_ENV_KEY).unwrap_or_default();
        let port = env::var(APP_PORT_ENV_KEY)
            .unwrap_or("3000".to_owned())
            .parse()
            .unwrap_or_default();
        let log_level = env::var(LOG_LEVEL_ENV_KEY).unwrap_or("debug".to_owned());

        let app_cfg = AppConfig {
            enable_external_creates_logging: false,
            env,
            host,
            log_level,
            name,
            port,
            secret_key,
        };

        self.app_cfg = app_cfg.clone();

        (app_cfg, self)
    }

    pub async fn build<T>(&mut self) -> Result<Configs<T>, ConfigsError>
    where
        T: DynamicConfig,
    {
        let client = self.get_secret_client().await?;
        self.client = Some(client);

        Ok(self.config())
    }
}

impl ConfigBuilder {
    async fn get_secret_client(&self) -> Result<Arc<dyn SecretClient>, ConfigsError> {
        let c: Arc<dyn SecretClient> = match (self.secret_client_kind, self.app_cfg.env.clone()) {
            (
                SecretClientKind::AWSSecreteManager,
                Environment::Prod | Environment::Staging | Environment::Dev,
            ) => {
                let secret_key = env::var(SECRET_KEY_ENV_KEY).unwrap_or_default();

                Arc::new(
                    AwsSecretClientBuilder::new()
                        .setup(self.app_cfg.env.to_string(), secret_key)
                        .build()
                        .await
                        .map_err(|e| {
                            error!(error = e.to_string(), "error to create aws secret client");
                            ConfigsError::SecretLoadingError(e.to_string())
                        })?,
                )
            }
            (_, _) => Arc::new(DummyClient::new()),
        };

        Ok(c)
    }

    fn config<T>(&self) -> Configs<T>
    where
        T: DynamicConfig,
    {
        let mut cfg = Configs::default();
        cfg.app = self.app_cfg.clone();

        for (key, value) in env::vars() {
            match key.as_str() {
                APP_NAME_ENV_KEY if cfg.app.name.is_empty() => {
                    cfg.app.name = self.get_string_from_secret(value, "name".to_owned());
                }
                SECRET_KEY_ENV_KEY if cfg.app.secret_key.is_empty() => {
                    cfg.app.secret_key = self.get_string_from_secret(value, "secret".to_owned());
                }
                HOST_NAME_ENV_KEY if cfg.app.host.is_empty() => {
                    cfg.app.host = self.get_string_from_secret(value, "localhost".to_owned());
                }
                APP_PORT_ENV_KEY if cfg.app.name.is_empty() => {
                    cfg.app.port = self.get_u64_from_secret(value, 31033);
                }
                LOG_LEVEL_ENV_KEY if cfg.app.log_level.is_empty() => {
                    cfg.app.log_level = self.get_string_from_secret(value, "debug".to_owned());
                }
                MQTT_HOST_ENV_KEY if self.mqtt => {
                    cfg.mqtt.host = self.get_string_from_secret(value, "localhost".to_owned());
                }
                MQTT_PORT_ENV_KEY if self.mqtt => {
                    cfg.mqtt.port = self.get_u64_from_secret(value, 1883);
                }
                MQTT_USER_ENV_KEY if self.mqtt => {
                    cfg.mqtt.user = self.get_string_from_secret(value, "mqtt".to_owned());
                }
                MQTT_PASSWORD_ENV_KEY if self.mqtt => {
                    cfg.mqtt.password = self.get_string_from_secret(value, "password".to_owned());
                }
                AMQP_HOST_ENV_KEY if self.amqp => {
                    cfg.amqp.host = self.get_string_from_secret(value, "localhost".to_owned());
                }
                AMQP_PORT_ENV_KEY if self.amqp => {
                    cfg.amqp.port = self.get_u64_from_secret(value, 5672);
                }
                AMQP_USER_ENV_KEY if self.amqp => {
                    cfg.amqp.user = self.get_string_from_secret(value, "guest".to_owned());
                }
                AMQP_PASSWORD_ENV_KEY if self.amqp => {
                    cfg.amqp.password = self.get_string_from_secret(value, "guest".to_owned());
                }
                AMQP_VHOST_ENV_KEY if self.amqp => {
                    cfg.amqp.vhost = self.get_string_from_secret(value, "".to_owned());
                }
                ENABLE_TRACES_ENV_KEY if self.otlp => {
                    cfg.otlp.enable_traces = self.get_bool_from_secret(value);
                }
                ENABLE_METRICS_ENV_KEY if self.otlp => {
                    cfg.otlp.enable_metrics = self.get_bool_from_secret(value);
                }
                OTLP_HOST_ENV_KEY if self.otlp => {
                    cfg.otlp.host = self.get_string_from_secret(value, "localhost".to_owned());
                }
                OTLP_ACCESS_KEY_ENV_KEY if self.otlp => {
                    cfg.otlp.key = self.get_string_from_secret(value, "key".to_owned());
                }
                OTLP_SERVICE_TYPE_ENV_KEY if self.otlp => {
                    cfg.otlp.service_type =
                        self.get_string_from_secret(value, "service".to_owned());
                }
                OTLP_EXPORT_TIMEOUT_ENV_KEY if self.otlp => {
                    cfg.otlp.export_timeout = self.get_u64_from_secret(value.clone(), 30);
                    cfg.otlp.metrics_export_interval = self.get_u64_from_secret(value, 60);
                }
                POSTGRES_HOST_ENV_KEY if self.postgres => {
                    cfg.postgres.host = self.get_string_from_secret(value, "localhost".to_owned());
                }
                POSTGRES_USER_ENV_KEY if self.postgres => {
                    cfg.postgres.user = self.get_string_from_secret(value, "postgres".to_owned());
                }
                POSTGRES_PASSWORD_ENV_KEY if self.postgres => {
                    cfg.postgres.password =
                        self.get_string_from_secret(value, "postgres".to_owned());
                }
                POSTGRES_PORT_ENV_KEY if self.postgres => {
                    cfg.postgres.port = self.get_u16_from_secret(value, 5432);
                }
                POSTGRES_DB_ENV_KEY => {
                    if self.postgres {
                        cfg.postgres.db = self.get_string_from_secret(value, "hdr".to_owned());
                    }
                }
                DYNAMO_ENDPOINT_ENV_KEY if self.dynamo => {
                    cfg.dynamo.endpoint =
                        self.get_string_from_secret(value, "localhost".to_owned());
                }
                DYNAMO_TABLE_ENV_KEY if self.dynamo => {
                    cfg.dynamo.table = self.get_string_from_secret(value, "table".to_owned());
                }
                DYNAMO_REGION_ENV_KEY if self.dynamo => {
                    let region = self.get_string_from_secret(value, AWS_DEFAULT_REGION.to_owned());
                    cfg.dynamo.region = region.clone();
                }
                CUSTOM_AWS_ACCESS_KEY_ID_ENV_KEY if self.aws => {
                    cfg.aws.access_key_id = self.get_string_from_secret(value, "key".to_owned());
                }
                CUSTOM_AWS_SECRET_ACCESS_KEY if self.aws => {
                    cfg.aws.secret_access_key =
                        self.get_string_from_secret(value, "secret".to_owned());
                }
                HEALTH_READINESS_PORT_ENV_KEY if self.health => {
                    cfg.health_readiness.port = self.get_u64_from_secret(value, 8888);
                }
                ENABLE_HEALTH_READINESS_ENV_KEY if self.health => {
                    cfg.health_readiness.enable = self.get_bool_from_secret(value);
                }
                SQLITE_FILE_NAME_ENV_KEY if self.sqlite => {
                    cfg.sqlite.file = self.get_string_from_secret(value, "local.db".to_owned());
                }
                "SQLITE_USER" if self.sqlite => {
                    cfg.sqlite.user = self.get_string_from_secret(value, "user".to_owned());
                }
                "SQLITE_PASSWORD" if self.sqlite => {
                    cfg.sqlite.password = self.get_string_from_secret(value, "password".to_owned());
                }
                _ => {}
            }
        }

        cfg
    }
}

impl ConfigBuilder {
    fn get_string_from_secret(&self, key: String, default: String) -> String {
        if !key.starts_with(SECRET_PREFIX) {
            return key;
        }

        let Ok(v) = self.client.clone().unwrap().get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        if !key.starts_with(SECRET_PREFIX_TO_DECODE) {
            return v;
        }

        self.decoded(v).unwrap_or_else(|_| {
            error!(key = key, "decoded went wrong");
            return default;
        })
    }

    fn get_u64_from_secret(&self, key: String, default: u64) -> u64 {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(default);
        }

        let Ok(v) = self.client.clone().unwrap().get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return default;
        })
    }

    fn _get_i32_from_secret(&self, key: String, default: i32) -> i32 {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(default);
        }

        let Ok(v) = self.client.clone().unwrap().get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return default;
        })
    }

    fn get_u16_from_secret(&self, key: String, default: u16) -> u16 {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(default);
        }

        let Ok(v) = self.client.clone().unwrap().get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return default;
        })
    }

    fn get_bool_from_secret(&self, key: String) -> bool {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(false);
        }

        let Ok(v) = self.client.clone().unwrap().get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return false;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return false;
        })
    }

    fn decoded(&self, text: String) -> Result<String, ()> {
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
}
