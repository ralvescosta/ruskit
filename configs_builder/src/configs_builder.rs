use crate::{
    env_keys::{
        APP_NAME_ENV_KEY, APP_PORT_ENV_KEY, AWS_DEFAULT_REGION, AWS_IAM_ACCESS_KEY_ID,
        AWS_IAM_SECRET_ACCESS_KEY, DEV_ENV_FILE_NAME, DYNAMO_ENDPOINT_ENV_KEY,
        DYNAMO_EXPIRE_ENV_KEY, DYNAMO_REGION_ENV_KEY, DYNAMO_TABLE_ENV_KEY,
        ENABLE_HEALTH_READINESS_ENV_KEY, ENABLE_METRICS_ENV_KEY, ENABLE_TRACES_ENV_KEY,
        HEALTH_READINESS_PORT_ENV_KEY, HOST_NAME_ENV_KEY, IDENTITY_SERVER_AUDIENCE_ENV_KEY,
        IDENTITY_SERVER_CLIENT_ID_ENV_KEY, IDENTITY_SERVER_CLIENT_SECRET_ENV_KEY,
        IDENTITY_SERVER_GRANT_TYPE_ENV_KEY, IDENTITY_SERVER_ISSUER_ENV_KEY,
        IDENTITY_SERVER_REALM_ENV_KEY, IDENTITY_SERVER_URL_ENV_KEY, KAFKA_HOST_ENV_KEY,
        KAFKA_PASSWORD_ENV_KEY, KAFKA_PORT_ENV_KEY, KAFKA_SASL_MECHANISMS_ENV_KEY,
        KAFKA_SECURITY_PROTOCOL_ENV_KEY, KAFKA_TIMEOUT_ENV_KEY, KAFKA_USER_ENV_KEY,
        LOCAL_ENV_FILE_NAME, LOG_LEVEL_ENV_KEY, METRIC_ACCESS_KEY_ENV_KEY, METRIC_EXPORTER_ENV_KEY,
        METRIC_EXPORT_RATE_BASE_ENV_KEY, METRIC_EXPORT_TIMEOUT_ENV_KEY,
        METRIC_HEADER_ACCESS_KEY_ENV_KEY, METRIC_HOST_ENV_KEY, METRIC_SERVICE_TYPE_ENV_KEY,
        MQTT_BROKER_KIND_ENV_KEY, MQTT_CA_CERT_PATH_ENV_KEY, MQTT_HOST_ENV_KEY,
        MQTT_PASSWORD_ENV_KEY, MQTT_PORT_ENV_KEY, MQTT_TRANSPORT_ENV_KEY, MQTT_USER_ENV_KEY,
        POSTGRES_DB_ENV_KEY, POSTGRES_HOST_ENV_KEY, POSTGRES_PASSWORD_ENV_KEY,
        POSTGRES_PORT_ENV_KEY, POSTGRES_USER_ENV_KEY, PROD_FILE_NAME, RABBITMQ_HOST_ENV_KEY,
        RABBITMQ_PASSWORD_ENV_KEY, RABBITMQ_PORT_ENV_KEY, RABBITMQ_USER_ENV_KEY,
        RABBITMQ_VHOST_ENV_KEY, SECRET_KEY_ENV_KEY, SECRET_MANAGER_ENV_KEY, SECRET_PREFIX,
        SQLITE_FILE_NAME_ENV_KEY, STAGING_FILE_NAME, TRACE_ACCESS_KEY_ENV_KEY,
        TRACE_EXPORTER_ENV_KEY, TRACE_EXPORT_RATE_BASE_ENV_KEY, TRACE_EXPORT_TIMEOUT_ENV_KEY,
        TRACE_HEADER_ACCESS_KEY_ENV_KEY, TRACE_HOST_ENV_KEY, TRACE_SERVICE_TYPE_ENV_KEY,
    },
    errors::ConfigsError,
};
use base64::{engine::general_purpose, Engine};
use configs::{
    AppConfigs, Configs, DynamicConfigs, Environment, MQTTBrokerKind, MQTTTransport,
    MetricExporterKind, SecretsManagerKind, TraceExporterKind,
};
use dotenvy::from_filename;
use secrets_manager::{AWSSecretClientBuilder, FakeSecretClient, SecretClient};
use std::{env, str::FromStr, sync::Arc};
use tracing::error;

#[derive(Default)]
pub struct ConfigBuilder {
    client: Option<Arc<dyn SecretClient>>,
    mqtt: bool,
    rabbitmq: bool,
    kafka: bool,
    postgres: bool,
    sqlite: bool,
    aws: bool,
    dynamo: bool,
    metric: bool,
    trace: bool,
    health: bool,
    identity: bool,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn mqtt(mut self) -> Self {
        self.mqtt = true;
        self
    }

    pub fn rabbitmq(mut self) -> Self {
        self.rabbitmq = true;
        self
    }

    pub fn kafka(mut self) -> Self {
        self.kafka = true;
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

    pub fn metric(mut self) -> Self {
        self.metric = true;
        self
    }

    pub fn trace(mut self) -> Self {
        self.trace = true;
        self
    }

    pub fn health(mut self) -> Self {
        self.health = true;
        self
    }

    pub fn identity_server(mut self) -> Self {
        self.identity = true;
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

        cfg.dynamic.load();

        self.client = self.get_secret_client(&cfg.app).await?;

        for (key, value) in env::vars() {
            if self.fill_identity_server(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_mqtt(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_rabbitmq(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_kafka(&mut cfg, &key, &value) {
                continue;
            };
            if self.fill_trace(&mut cfg, &key, &value) {
                continue;
            }
            if self.fill_metric(&mut cfg, &key, &value) {
                continue;
            }
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
    ) -> Result<Option<Arc<dyn SecretClient>>, ConfigsError> {
        match app_cfg.secret_manager {
            SecretsManagerKind::None => Ok(Some(Arc::new(FakeSecretClient::new()))),

            SecretsManagerKind::AWSSecretManager => {
                let secret_key = env::var(SECRET_KEY_ENV_KEY).unwrap_or_default();

                match AWSSecretClientBuilder::new(app_cfg.env.to_string(), secret_key)
                    .build()
                    .await
                {
                    Ok(c) => Ok(Some(Arc::new(c))),
                    Err(err) => {
                        error!(error = err.to_string(), "error to create aws secret client");
                        Err(ConfigsError::SecretLoadingError(err.to_string()))
                    }
                }
            }
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
            .unwrap_or("3000".into())
            .parse()
            .unwrap_or_default();
        let log_level = env::var(LOG_LEVEL_ENV_KEY).unwrap_or("debug".into());
        let secret_manager = env::var(SECRET_MANAGER_ENV_KEY).unwrap_or("None".into());

        cfg.app = AppConfigs {
            enable_external_creates_logging: false,
            env,
            host,
            log_level,
            name,
            port,
            secret_key,
            secret_manager: SecretsManagerKind::from(&secret_manager),
        };
    }

    fn fill_metric<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            ENABLE_METRICS_ENV_KEY if self.metric => {
                cfg.metric.enable = self.get_from_secret(value.into(), false);
                true
            }
            METRIC_EXPORTER_ENV_KEY if self.metric => {
                cfg.metric.exporter =
                    self.get_from_secret(value.into(), MetricExporterKind::Stdout);
                true
            }
            METRIC_HOST_ENV_KEY if self.metric => {
                cfg.metric.host = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            METRIC_HEADER_ACCESS_KEY_ENV_KEY if self.metric => {
                cfg.metric.header_access_key = self.get_from_secret(value.into(), "api-key".into());
                true
            }
            METRIC_ACCESS_KEY_ENV_KEY if self.metric => {
                cfg.metric.access_key = self.get_from_secret(value.into(), "key".into());
                true
            }
            METRIC_SERVICE_TYPE_ENV_KEY if self.metric => {
                cfg.metric.service_type = self.get_from_secret(value.into(), "service".into());
                true
            }
            METRIC_EXPORT_TIMEOUT_ENV_KEY if self.metric => {
                let k: String = value.into();
                cfg.metric.export_timeout = self.get_from_secret(k.clone(), 30);
                true
            }
            METRIC_EXPORT_RATE_BASE_ENV_KEY if self.metric => {
                cfg.metric.export_rate_base = self.get_from_secret(value.into(), 0.8);
                true
            }
            _ => false,
        }
    }

    fn fill_trace<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            ENABLE_TRACES_ENV_KEY if self.trace => {
                cfg.trace.enable = self.get_from_secret(value.into(), false);
                true
            }
            TRACE_EXPORTER_ENV_KEY if self.trace => {
                cfg.trace.exporter = self.get_from_secret(value.into(), TraceExporterKind::Stdout);
                true
            }
            TRACE_HOST_ENV_KEY if self.trace => {
                cfg.trace.host = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            TRACE_HEADER_ACCESS_KEY_ENV_KEY if self.trace => {
                cfg.trace.header_access_key = self.get_from_secret(value.into(), "api-key".into());
                true
            }
            TRACE_ACCESS_KEY_ENV_KEY if self.trace => {
                cfg.trace.access_key = self.get_from_secret(value.into(), "key".into());
                true
            }
            TRACE_SERVICE_TYPE_ENV_KEY if self.trace => {
                cfg.trace.service_type = self.get_from_secret(value.into(), "service".into());
                true
            }
            TRACE_EXPORT_TIMEOUT_ENV_KEY if self.trace => {
                let k: String = value.into();
                cfg.trace.export_timeout = self.get_from_secret(k.clone(), 30);
                true
            }
            TRACE_EXPORT_RATE_BASE_ENV_KEY if self.trace => {
                cfg.trace.export_rate_base = self.get_from_secret(value.into(), 0.8);
                true
            }
            _ => false,
        }
    }

    fn fill_identity_server<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            IDENTITY_SERVER_URL_ENV_KEY if self.identity => {
                cfg.identity.url = self.get_from_secret(value.into(), "http://localhost".into());
                true
            }
            IDENTITY_SERVER_REALM_ENV_KEY if self.identity => {
                cfg.identity.realm = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            IDENTITY_SERVER_AUDIENCE_ENV_KEY if self.identity => {
                cfg.identity.audience = self.get_from_secret(value.into(), "audience".into());
                true
            }
            IDENTITY_SERVER_ISSUER_ENV_KEY if self.identity => {
                cfg.identity.issuer = self.get_from_secret(value.into(), "issuer".into());
                true
            }
            IDENTITY_SERVER_GRANT_TYPE_ENV_KEY if self.identity => {
                cfg.identity.grant_type =
                    self.get_from_secret(value.into(), "client_credentials".into());
                true
            }
            IDENTITY_SERVER_CLIENT_ID_ENV_KEY if self.identity => {
                cfg.identity.client_id = self.get_from_secret(value.into(), "".into());
                true
            }
            IDENTITY_SERVER_CLIENT_SECRET_ENV_KEY if self.identity => {
                cfg.identity.client_secret = self.get_from_secret(value.into(), "".into());
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
            MQTT_BROKER_KIND_ENV_KEY if self.mqtt => {
                let kind = self.get_from_secret::<String>(value.into(), "SELF_HOSTED".into());
                cfg.mqtt.broker_kind = MQTTBrokerKind::from(&kind);
                true
            }
            MQTT_HOST_ENV_KEY if self.mqtt => {
                cfg.mqtt.host = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            MQTT_TRANSPORT_ENV_KEY if self.mqtt => {
                let transport = self.get_from_secret::<String>(value.into(), "tcp".into());
                cfg.mqtt.transport = MQTTTransport::from(&transport);
                true
            }
            MQTT_PORT_ENV_KEY if self.mqtt => {
                cfg.mqtt.port = self.get_from_secret(value.into(), 1883);
                true
            }
            MQTT_USER_ENV_KEY if self.mqtt => {
                cfg.mqtt.user = self.get_from_secret(value.into(), "mqtt".into());
                true
            }
            MQTT_PASSWORD_ENV_KEY if self.mqtt => {
                cfg.mqtt.password = self.get_from_secret(value.into(), "password".into());
                true
            }
            MQTT_CA_CERT_PATH_ENV_KEY if self.mqtt => {
                cfg.mqtt.root_ca_path = self.get_from_secret(value.into(), "".into());
                true
            }
            _ => false,
        }
    }

    fn fill_rabbitmq<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            RABBITMQ_HOST_ENV_KEY if self.rabbitmq => {
                cfg.rabbitmq.host = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            RABBITMQ_PORT_ENV_KEY if self.rabbitmq => {
                cfg.rabbitmq.port = self.get_from_secret(value.into(), 5672);
                true
            }
            RABBITMQ_USER_ENV_KEY if self.rabbitmq => {
                cfg.rabbitmq.user = self.get_from_secret(value.into(), "guest".into());
                true
            }
            RABBITMQ_PASSWORD_ENV_KEY if self.rabbitmq => {
                cfg.rabbitmq.password = self.get_from_secret(value.into(), "guest".into());
                true
            }
            RABBITMQ_VHOST_ENV_KEY if self.rabbitmq => {
                cfg.rabbitmq.vhost = self.get_from_secret(value.into(), "".into());
                true
            }
            _ => false,
        }
    }

    fn fill_kafka<T>(
        &self,
        cfg: &mut Configs<T>,
        key: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> bool
    where
        T: DynamicConfigs,
    {
        match key.into().as_str() {
            KAFKA_HOST_ENV_KEY if self.kafka => {
                cfg.kafka.host = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            KAFKA_PORT_ENV_KEY if self.kafka => {
                cfg.kafka.port = self.get_from_secret(value.into(), 9094);
                true
            }
            KAFKA_TIMEOUT_ENV_KEY if self.kafka => {
                cfg.kafka.timeout = self.get_from_secret(value.into(), 6000);
                true
            }
            KAFKA_SECURITY_PROTOCOL_ENV_KEY if self.kafka => {
                cfg.kafka.security_protocol = self.get_from_secret(value.into(), "SASL_SSL".into());
                true
            }
            KAFKA_SASL_MECHANISMS_ENV_KEY if self.kafka => {
                cfg.kafka.sasl_mechanisms = self.get_from_secret(value.into(), "PLAIN".into());
                true
            }
            KAFKA_USER_ENV_KEY if self.kafka => {
                cfg.kafka.user = self.get_from_secret(value.into(), "user".into());
                true
            }
            KAFKA_PASSWORD_ENV_KEY if self.kafka => {
                cfg.kafka.password = self.get_from_secret(value.into(), "password".into());
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
                cfg.postgres.host = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            POSTGRES_USER_ENV_KEY if self.postgres => {
                cfg.postgres.user = self.get_from_secret(value.into(), "postgres".into());
                true
            }
            POSTGRES_PASSWORD_ENV_KEY if self.postgres => {
                cfg.postgres.password = self.get_from_secret(value.into(), "postgres".into());
                true
            }
            POSTGRES_PORT_ENV_KEY if self.postgres => {
                cfg.postgres.port = self.get_from_secret(value.into(), 5432);
                true
            }
            POSTGRES_DB_ENV_KEY if self.postgres => {
                cfg.postgres.db = self.get_from_secret(value.into(), "hdr".into());
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
                cfg.dynamo.endpoint = self.get_from_secret(value.into(), "localhost".into());
                true
            }
            DYNAMO_TABLE_ENV_KEY if self.dynamo => {
                cfg.dynamo.table = self.get_from_secret(value.into(), "table".into());
                true
            }
            DYNAMO_REGION_ENV_KEY if self.dynamo => {
                cfg.dynamo.region = self.get_from_secret(value.into(), AWS_DEFAULT_REGION.into());
                true
            }
            DYNAMO_EXPIRE_ENV_KEY if self.dynamo => {
                cfg.dynamo.expire = self.get_from_secret(value.into(), 31536000);
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
                cfg.aws.access_key_id = Some(self.get_from_secret(value.into(), "key".into()));
                true
            }
            AWS_IAM_SECRET_ACCESS_KEY if self.aws => {
                cfg.aws.secret_access_key =
                    Some(self.get_from_secret(value.into(), "secret".into()));
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
                cfg.sqlite.file = self.get_from_secret(value.into(), "local.db".into());
                true
            }
            "SQLITE_USER" if self.sqlite => {
                cfg.sqlite.user = self.get_from_secret(value.into(), "user".into());
                true
            }
            "SQLITE_PASSWORD" if self.sqlite => {
                cfg.sqlite.password = self.get_from_secret(value.into(), "password".into());
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
            default
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
