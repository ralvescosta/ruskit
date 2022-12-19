use crate::{Config, Environment, SECRET_PREFIX, SECRET_PREFIX_TO_DECODE};
use dotenv::from_filename;
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
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn use_aws_secrets(mut self) -> Self {
        self.secret_client_kind = SecretClientKind::AWSSecreteManager;
        self
    }
}

impl ConfigBuilder {
    pub fn load_from_file(&self, env: &Environment) {
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
    }

    fn get_string_from_secret(
        &self,
        key: String,
        c: Arc<dyn SecretClient>,
        default: String,
    ) -> String {
        if !key.starts_with(SECRET_PREFIX) {
            return key;
        }

        let Ok(v) = c.get_by_key(&key) else {
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

    fn get_u64_from_secret(&self, key: String, c: Arc<dyn SecretClient>, default: u64) -> u64 {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(default);
        }

        let Ok(v) = c.get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return default;
        })
    }

    fn get_i32_from_secret(&self, key: String, c: Arc<dyn SecretClient>, default: i32) -> i32 {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(default);
        }

        let Ok(v) = c.get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return default;
        })
    }

    fn get_u16_from_secret(&self, key: String, c: Arc<dyn SecretClient>, default: u16) -> u16 {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(default);
        }

        let Ok(v) = c.get_by_key(&key) else {
          error!(key = key, "secret key was not found");
          return default;
        };

        v.parse().unwrap_or_else(|_| {
            error!(key = key, value = v, "parse went wrong");
            return default;
        })
    }

    fn get_bool_from_secret(&self, key: String, c: Arc<dyn SecretClient>) -> bool {
        if !key.starts_with(SECRET_PREFIX) {
            return key.parse().unwrap_or(false);
        }

        let Ok(v) = c.get_by_key(&key) else {
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

    async fn secret_client(&self) -> Result<(Environment, Arc<dyn SecretClient>), ConfigsError> {
        let env = Environment::from_rust_env();

        self.load_from_file(&env);

        let c: Arc<dyn SecretClient> = match (self.secret_client_kind, env) {
            (
                SecretClientKind::AWSSecreteManager,
                Environment::Prod | Environment::Staging | Environment::Dev,
            ) => {
                let app_ctx = env::var("APP_CONTEXT").unwrap_or_default();

                Arc::new(
                    AwsSecretClientBuilder::new()
                        .setup(env.to_string(), app_ctx)
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

        Ok((env, c))
    }

    pub async fn build(&self) -> Result<Config, ConfigsError> {
        let (env, client) = self.secret_client().await?;

        Ok(self.config(env, client))
    }
}

impl ConfigBuilder {
    fn config(&self, env: Environment, client: Arc<dyn SecretClient>) -> Config {
        let mut cfg = Config::default();
        cfg.app.env = env;
        cfg.app.enable_external_creates_logging = false;

        for (key, value) in env::vars() {
            match key.as_str() {
                "APP_NAME" => {
                    cfg.app.name =
                        self.get_string_from_secret(value, client.clone(), "name".to_owned());
                }
                "APP_CONTEXT" => {
                    cfg.app.ctx =
                        self.get_string_from_secret(value, client.clone(), "context".to_owned());
                }
                "HOST_NAME" => {
                    cfg.app.host =
                        self.get_string_from_secret(value, client.clone(), "localhost".to_owned());
                }
                "APP_PORT" => {
                    cfg.app.port = self.get_u64_from_secret(value, client.clone(), 31033);
                }
                "LOG_LEVEL" => {
                    cfg.app.log_level =
                        self.get_string_from_secret(value, client.clone(), "debug".to_owned());
                }
                "MQTT_HOST" => {
                    cfg.mqtt.host =
                        self.get_string_from_secret(value, client.clone(), "localhost".to_owned());
                }
                "MQTT_PORT" => {
                    cfg.mqtt.port = self.get_u64_from_secret(value, client.clone(), 1883);
                }
                "MQTT_USER" => {
                    cfg.mqtt.user =
                        self.get_string_from_secret(value, client.clone(), "mqtt".to_owned());
                }
                "MQTT_PASSWORD" => {
                    cfg.mqtt.password =
                        self.get_string_from_secret(value, client.clone(), "password".to_owned());
                }
                "AMQP_HOST" => {
                    cfg.amqp.host =
                        self.get_string_from_secret(value, client.clone(), "localhost".to_owned());
                }
                "AMQP_PORT" => {
                    cfg.amqp.port = self.get_u64_from_secret(value, client.clone(), 5672);
                }
                "AMQP_USER" => {
                    cfg.amqp.user =
                        self.get_string_from_secret(value, client.clone(), "guest".to_owned());
                }
                "AMQP_PASSWORD" => {
                    cfg.amqp.password =
                        self.get_string_from_secret(value, client.clone(), "guest".to_owned());
                }
                "AMQP_VHOST" => {
                    cfg.amqp.vhost =
                        self.get_string_from_secret(value, client.clone(), "".to_owned());
                }
                "ENABLE_TRACES" => {
                    cfg.otlp.enable_traces = self.get_bool_from_secret(value, client.clone());
                }
                "ENABLE_METRICS" => {
                    cfg.otlp.enable_metrics = self.get_bool_from_secret(value, client.clone());
                }
                "OTLP_HOST" => {
                    cfg.otlp.host =
                        self.get_string_from_secret(value, client.clone(), "localhost".to_owned());
                }
                "OTLP_KEY" => {
                    cfg.otlp.key =
                        self.get_string_from_secret(value, client.clone(), "key".to_owned());
                }
                "OTLP_SERVICE_TYPE" => {
                    cfg.otlp.service_type =
                        self.get_string_from_secret(value, client.clone(), "service".to_owned());
                }
                "OTLP_EXPORT_TIMEOUT" => {
                    cfg.otlp.export_timeout =
                        self.get_u64_from_secret(value.clone(), client.clone(), 30);
                    cfg.otlp.metrics_export_interval =
                        self.get_u64_from_secret(value, client.clone(), 60);
                }
                "POSTGRES_HOST" => {
                    cfg.postgres.host =
                        self.get_string_from_secret(value, client.clone(), "localhost".to_owned());
                }
                "POSTGRES_USER" => {
                    cfg.postgres.user =
                        self.get_string_from_secret(value, client.clone(), "postgres".to_owned());
                }
                "POSTGRES_PASSWORD" => {
                    cfg.postgres.password =
                        self.get_string_from_secret(value, client.clone(), "postgres".to_owned());
                }
                "POSTGRES_PORT" => {
                    cfg.postgres.port = self.get_u16_from_secret(value, client.clone(), 5432);
                }
                "POSTGRES_DB" => {
                    cfg.postgres.db =
                        self.get_string_from_secret(value, client.clone(), "hdr".to_owned());
                }
                "DYNAMO_HOST" => {
                    cfg.dynamo.host =
                        self.get_string_from_secret(value, client.clone(), "localhost".to_owned());
                }
                "DYNAMO_USER" => {
                    cfg.dynamo.user =
                        self.get_string_from_secret(value, client.clone(), "user".to_owned());
                }
                "DYNAMO_PASSWORD" => {
                    cfg.dynamo.password =
                        self.get_string_from_secret(value, client.clone(), "password".to_owned());
                }
                "HEALTH_READINESS_PORT" => {
                    cfg.health_readiness.port =
                        self.get_u64_from_secret(value, client.clone(), 8888);
                }
                "ENABLE_HEALTH_READINESS" => {
                    cfg.health_readiness.enable = self.get_bool_from_secret(value, client.clone());
                }
                "MULTIPLE_MESSAGE_TIMER" => {
                    cfg.multiple_message_timer =
                        self.get_i32_from_secret(value, client.clone(), 15000);
                }
                _ => {}
            }
        }

        cfg
    }
}
