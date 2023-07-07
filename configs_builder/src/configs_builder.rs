use crate::{
    env_keys::{
        AMQP_HOST_ENV_KEY, AMQP_PASSWORD_ENV_KEY, AMQP_PORT_ENV_KEY, AMQP_USER_ENV_KEY,
        AMQP_VHOST_ENV_KEY, APP_NAME_ENV_KEY, APP_PORT_ENV_KEY, AUTH0_AUDIENCE_ENV_KEY,
        AUTH0_CLIENT_ID_ENV_KEY, AUTH0_CLIENT_SECRET_ENV_KEY, AUTH0_DOMAIN_ENV_KEY,
        AUTH0_GRANT_TYPE_ENV_KEY, AUTH0_ISSUER_ENV_KEY, AWS_DEFAULT_REGION, AWS_IAM_ACCESS_KEY_ID,
        AWS_IAM_SECRET_ACCESS_KEY, DEV_ENV_FILE_NAME, DYNAMO_ENDPOINT_ENV_KEY,
        DYNAMO_REGION_ENV_KEY, DYNAMO_TABLE_ENV_KEY, ENABLE_HEALTH_READINESS_ENV_KEY,
        ENABLE_METRICS_ENV_KEY, ENABLE_TRACES_ENV_KEY, HEALTH_READINESS_PORT_ENV_KEY,
        HOST_NAME_ENV_KEY, LOCAL_ENV_FILE_NAME, LOG_LEVEL_ENV_KEY, MQTT_CA_CERT_PATH_ENV_KEY,
        MQTT_HOST_ENV_KEY, MQTT_PASSWORD_ENV_KEY, MQTT_PORT_ENV_KEY, MQTT_TRANSPORT_ENV_KEY,
        MQTT_USER_ENV_KEY, OTLP_ACCESS_KEY_ENV_KEY, OTLP_EXPORT_RATE_BASE_ENV_KEY,
        OTLP_EXPORT_TIMEOUT_ENV_KEY, OTLP_HOST_ENV_KEY, OTLP_SERVICE_TYPE_ENV_KEY,
        POSTGRES_DB_ENV_KEY, POSTGRES_HOST_ENV_KEY, POSTGRES_PASSWORD_ENV_KEY,
        POSTGRES_PORT_ENV_KEY, POSTGRES_USER_ENV_KEY, PROD_FILE_NAME, SECRET_KEY_ENV_KEY,
        SECRET_PREFIX, SQLITE_FILE_NAME_ENV_KEY, STAGING_FILE_NAME, USE_SECRET_MANAGER_ENV_KEY,
    },
    errors::ConfigsError,
};
use base64::{engine::general_purpose, Engine};
use configs::{AppConfigs, Configs, DynamicConfigs, Environment};
use dotenvy::from_filename;
use secrets_manager::{AWSSecretClientBuilder, DummyClient, SecretClient};
use std::{env, str::FromStr, sync::Arc};
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
    mqtt: bool,
    amqp: bool,
    postgres: bool,
    sqlite: bool,
    aws: bool,
    dynamo: bool,
    otlp: bool,
    health: bool,
    auth0: bool,
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

    pub fn auth0(mut self) -> Self {
        self.auth0 = true;
        self
    }

    pub fn use_aws_secret_manager(mut self) -> Self {
        self.secret_client_kind = SecretClientKind::AWSSecreteManager;
        self
    }

    pub async fn build<'c, T>(&mut self) -> Result<Configs<T>, ConfigsError>
    where
        T: DynamicConfigs,
    {
        let env = Environment::from_rust_env();
        match env {
            Environment::Prod => {
                from_filename(PROD_FILE_NAME).ok();
            }
            Environment::Staging => {
                from_filename(STAGING_FILE_NAME).ok();
            }
            Environment::Dev => {
                from_filename(DEV_ENV_FILE_NAME).ok();
            }
            _ => {
                from_filename(LOCAL_ENV_FILE_NAME).ok();
            }
        }

        let mut cfg = Configs::<T>::default();
        self.fill_app(&mut cfg);

        match logging::setup(&cfg.app) {
            Err(_) => Err(ConfigsError::InternalError {}),
            _ => Ok(()),
        }?;

        self.client = Some(self.get_secret_client(&cfg.app).await?);
        for (key, value) in env::vars() {
            if self.fill_auth0(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_mqtt(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_amqp(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_otlp(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_postgres(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_dynamo(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_aws(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_health_readiness(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_sqlite(&mut cfg, &key, &value) {
                continue;
            };
        }

        Ok(cfg)
    }
}

impl ConfigBuilder {
    async fn get_secret_client(
        &self,
        app_cfg: &AppConfigs,
    ) -> Result<Arc<dyn SecretClient>, ConfigsError> {
        if !app_cfg.use_secret_manager {
            return Ok(Arc::new(DummyClient::new()));
        }

        match self.secret_client_kind {
            SecretClientKind::AWSSecreteManager => {
                let secret_key = env::var(SECRET_KEY_ENV_KEY).unwrap_or_default();

                match AWSSecretClientBuilder::new(app_cfg.env.to_string(), secret_key)
                    .build()
                    .await
                {
                    Ok(c) => Ok(Arc::new(c)),
                    Err(err) => {
                        error!(error = err.to_string(), "error to create aws secret client");
                        Err(ConfigsError::SecretLoadingError(err.to_string()))
                    }
                }
            }
            _ => Err(ConfigsError::SecretLoadingError(
                "secret manager not provided".to_owned(),
            )),
        }
    }
}

impl ConfigBuilder {
    fn fill_app<T>(&self, cfg: &mut Configs<T>)
    where
        T: DynamicConfigs,
    {
        let env = Environment::from_rust_env();
        let name = self.fmt_name(&env, env::var(APP_NAME_ENV_KEY).unwrap_or_default());
        let secret_key = env::var(SECRET_KEY_ENV_KEY).unwrap_or_default();
        let host = env::var(HOST_NAME_ENV_KEY).unwrap_or_default();
        let port = env::var(APP_PORT_ENV_KEY)
            .unwrap_or("3000".to_owned())
            .parse()
            .unwrap_or_default();
        let log_level = env::var(LOG_LEVEL_ENV_KEY).unwrap_or("debug".to_owned());
        let use_secret_manager = env::var(USE_SECRET_MANAGER_ENV_KEY)
            .unwrap_or("false".to_owned())
            .parse()
            .unwrap();

        cfg.app = AppConfigs {
            enable_external_creates_logging: false,
            env,
            host,
            log_level,
            name,
            port,
            secret_key,
            use_secret_manager,
        };
    }

    fn fill_auth0<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            AUTH0_DOMAIN_ENV_KEY if self.auth0 => {
                cfg.auth0.domain = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            AUTH0_AUDIENCE_ENV_KEY if self.auth0 => {
                cfg.auth0.audience = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            AUTH0_ISSUER_ENV_KEY if self.auth0 => {
                cfg.auth0.issuer = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            AUTH0_GRANT_TYPE_ENV_KEY if self.auth0 => {
                cfg.auth0.grant_type =
                    self.get_from_secret(value.into(), "client_credentials".to_owned());
                true
            }
            AUTH0_CLIENT_ID_ENV_KEY if self.auth0 => {
                cfg.auth0.client_id = self.get_from_secret(value.into(), "".to_owned());
                true
            }
            AUTH0_CLIENT_SECRET_ENV_KEY if self.auth0 => {
                cfg.auth0.client_secret = self.get_from_secret(value.into(), "".to_owned());
                true
            }
            _ => false,
        }
    }

    fn fill_mqtt<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            MQTT_HOST_ENV_KEY if self.mqtt => {
                cfg.mqtt.host = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            MQTT_TRANSPORT_ENV_KEY if self.mqtt => {
                cfg.mqtt.transport = self.get_from_secret(value.into(), "tcp".to_owned());
                true
            }
            MQTT_PORT_ENV_KEY if self.mqtt => {
                cfg.mqtt.port = self.get_from_secret(value.into(), 1883);
                true
            }
            MQTT_USER_ENV_KEY if self.mqtt => {
                cfg.mqtt.user = self.get_from_secret(value.into(), "mqtt".to_owned());
                true
            }
            MQTT_PASSWORD_ENV_KEY if self.mqtt => {
                cfg.mqtt.password = self.get_from_secret(value.into(), "password".to_owned());
                true
            }
            MQTT_CA_CERT_PATH_ENV_KEY if self.mqtt => {
                cfg.mqtt.root_ca_path = self.get_from_secret(value.into(), "".to_owned());
                true
            }
            _ => false,
        }
    }

    fn fill_amqp<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            AMQP_HOST_ENV_KEY if self.amqp => {
                cfg.amqp.host = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            AMQP_PORT_ENV_KEY if self.amqp => {
                cfg.amqp.port = self.get_from_secret(value.into(), 5672);
                true
            }
            AMQP_USER_ENV_KEY if self.amqp => {
                cfg.amqp.user = self.get_from_secret(value.into(), "guest".to_owned());
                true
            }
            AMQP_PASSWORD_ENV_KEY if self.amqp => {
                cfg.amqp.password = self.get_from_secret(value.into(), "guest".to_owned());
                true
            }
            AMQP_VHOST_ENV_KEY if self.amqp => {
                cfg.amqp.vhost = self.get_from_secret(value.into(), "".to_owned());
                true
            }
            _ => false,
        }
    }

    fn fill_otlp<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            ENABLE_TRACES_ENV_KEY if self.otlp => {
                cfg.otlp.enable_traces = self.get_from_secret(value.into(), false);
                true
            }
            ENABLE_METRICS_ENV_KEY if self.otlp => {
                cfg.otlp.enable_metrics = self.get_from_secret(value.into(), false);
                true
            }
            OTLP_HOST_ENV_KEY if self.otlp => {
                cfg.otlp.host = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            OTLP_ACCESS_KEY_ENV_KEY if self.otlp => {
                cfg.otlp.key = self.get_from_secret(value.into(), "key".to_owned());
                true
            }
            OTLP_SERVICE_TYPE_ENV_KEY if self.otlp => {
                cfg.otlp.service_type = self.get_from_secret(value.into(), "service".to_owned());
                true
            }
            OTLP_EXPORT_TIMEOUT_ENV_KEY if self.otlp => {
                let k: String = value.into();
                cfg.otlp.export_timeout = self.get_from_secret(k.clone(), 30);
                cfg.otlp.metrics_export_interval = self.get_from_secret(k, 60);
                true
            }
            OTLP_EXPORT_RATE_BASE_ENV_KEY if self.otlp => {
                cfg.otlp.export_rate_base = self.get_from_secret(value.into(), 0.8);
                true
            }
            _ => false,
        }
    }

    fn fill_postgres<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            POSTGRES_HOST_ENV_KEY if self.postgres => {
                cfg.postgres.host = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            POSTGRES_USER_ENV_KEY if self.postgres => {
                cfg.postgres.user = self.get_from_secret(value.into(), "postgres".to_owned());
                true
            }
            POSTGRES_PASSWORD_ENV_KEY if self.postgres => {
                cfg.postgres.password = self.get_from_secret(value.into(), "postgres".to_owned());
                true
            }
            POSTGRES_PORT_ENV_KEY if self.postgres => {
                cfg.postgres.port = self.get_from_secret(value.into(), 5432);
                true
            }
            POSTGRES_DB_ENV_KEY if self.postgres => {
                cfg.postgres.db = self.get_from_secret(value.into(), "hdr".to_owned());
                true
            }
            _ => false,
        }
    }

    fn fill_dynamo<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            DYNAMO_ENDPOINT_ENV_KEY if self.dynamo => {
                cfg.dynamo.endpoint = self.get_from_secret(value.into(), "localhost".to_owned());
                true
            }
            DYNAMO_TABLE_ENV_KEY if self.dynamo => {
                cfg.dynamo.table = self.get_from_secret(value.into(), "table".to_owned());
                true
            }
            DYNAMO_REGION_ENV_KEY if self.dynamo => {
                cfg.dynamo.region =
                    self.get_from_secret(value.into(), AWS_DEFAULT_REGION.to_owned());
                true
            }
            _ => false,
        }
    }

    fn fill_aws<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            AWS_IAM_ACCESS_KEY_ID if self.aws => {
                cfg.aws.access_key_id = Some(self.get_from_secret(value.into(), "key".to_owned()));
                true
            }
            AWS_IAM_SECRET_ACCESS_KEY if self.aws => {
                cfg.aws.secret_access_key =
                    Some(self.get_from_secret(value.into(), "secret".to_owned()));
                true
            }
            _ => false,
        }
    }

    fn fill_health_readiness<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            HEALTH_READINESS_PORT_ENV_KEY if self.health => {
                cfg.health_readiness.port = self.get_from_secret(value.into(), 8888);
                true
            }
            ENABLE_HEALTH_READINESS_ENV_KEY if self.health => {
                cfg.health_readiness.enable = self.get_from_secret(value.into(), false);
                true
            }
            _ => false,
        }
    }

    fn fill_sqlite<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            SQLITE_FILE_NAME_ENV_KEY if self.sqlite => {
                cfg.sqlite.file = self.get_from_secret(value.into(), "local.db".to_owned());
                true
            }
            "SQLITE_USER" if self.sqlite => {
                cfg.sqlite.user = self.get_from_secret(value.into(), "user".to_owned());
                true
            }
            "SQLITE_PASSWORD" if self.sqlite => {
                cfg.sqlite.password = self.get_from_secret(value.into(), "password".to_owned());
                true
            }
            _ => false,
        }
    }
}

impl ConfigBuilder {
    fn get_from_secret<T>(&self, key: String, default: T) -> T
    where
        T: FromStr,
    {
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

    fn fmt_name(&self, env: &Environment, name: String) -> String {
        let env_str = env.to_string();
        if name.starts_with(&env_str) {
            return name;
        }

        format!("{}-{}", env_str, name)
    }

    fn _decoded(&self, text: String) -> Result<String, ()> {
        let d = match general_purpose::STANDARD.decode(text) {
            Err(err) => {
                error!(error = err.to_string(), "base64 decoded error");
                Err(())
            }
            Ok(v) => Ok(v),
        }?;

        match String::from_utf8(d) {
            Err(err) => {
                error!(error = err.to_string(), "error to convert to String");
                Err(())
            }
            Ok(s) => Ok(s),
        }
    }
}
