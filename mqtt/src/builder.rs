use crate::errors::MQTTError;
use configs::{AppConfigs, Configs, DynamicConfigs, MQTTConfigs};
use core::fmt;
use paho_mqtt::{
    AsyncClient, AsyncReceiver, ConnectOptions, ConnectOptionsBuilder, CreateOptions,
    CreateOptionsBuilder, Message, SslOptionsBuilder, SslVersion,
};
use std::{sync::Arc, time::Duration};
use tracing::error;

#[derive(Clone, PartialEq, Eq, Default)]
pub enum TransportKind {
    #[default]
    TCP,
    SSL,
}

impl fmt::Display for TransportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportKind::TCP => write!(f, "tcp"),
            TransportKind::SSL => write!(f, "ssl"),
        }
    }
}

#[derive(Clone, Default)]
pub enum ConnectionKind {
    #[default]
    WithPassword,
    WithoutPassword,
    AWSIoTCore,
}

pub struct MQTTClientBuilder {
    mqtt_cfg: MQTTConfigs,
    app_cfg: AppConfigs,
    transport_kind: TransportKind,
    connection_kind: ConnectionKind,
}

impl MQTTClientBuilder {
    pub fn new() -> MQTTClientBuilder {
        MQTTClientBuilder {
            mqtt_cfg: MQTTConfigs::default(),
            app_cfg: AppConfigs::default(),
            transport_kind: TransportKind::TCP,
            connection_kind: ConnectionKind::WithPassword,
        }
    }

    pub fn cfg<T>(mut self, cfgs: &Configs<T>) -> Self
    where
        T: DynamicConfigs,
    {
        self.mqtt_cfg = cfgs.mqtt.clone();
        self.app_cfg = cfgs.app.clone();

        if self.mqtt_cfg.transport == "ssl" {
            self.transport_kind = TransportKind::SSL
        } else {
            self.transport_kind = TransportKind::TCP
        }

        return self;
    }

    pub fn use_aws_iot_core(mut self) -> Self {
        self.connection_kind = ConnectionKind::AWSIoTCore;
        return self;
    }

    pub fn use_password(mut self) -> Self {
        self.connection_kind = ConnectionKind::WithPassword;
        return self;
    }

    pub fn use_password_less(mut self) -> Self {
        self.connection_kind = ConnectionKind::WithoutPassword;
        return self;
    }

    pub async fn build(
        self,
    ) -> Result<(Arc<AsyncClient>, AsyncReceiver<Option<Message>>), MQTTError> {
        if self.transport_kind == TransportKind::SSL && self.mqtt_cfg.root_ca_path.is_empty() {
            return Err(MQTTError::SSLMustContainCACertError {});
        }

        let crate_opts = match self.connection_kind {
            ConnectionKind::AWSIoTCore => self.aws_iot_core_crate_opts(),
            ConnectionKind::WithPassword => self.default_crate_opts(),
            ConnectionKind::WithoutPassword => self.default_crate_opts(),
        };

        let conn_opts = match self.connection_kind {
            ConnectionKind::WithoutPassword => self.password_less_connection_opts(),
            ConnectionKind::WithPassword => self.password_connection_opts(),
            ConnectionKind::AWSIoTCore => self.aws_iot_core_connection_opts(),
        };

        let mut client = match AsyncClient::new(crate_opts) {
            Err(err) => {
                error!(error = err.to_string(), "error to create mqtt client");
                Err(MQTTError::ConnectionError {})
            }
            Ok(c) => Ok(c),
        }?;

        let stream = client.get_stream(2048);

        match client.connect(conn_opts.clone()).await {
            Err(err) => {
                error!(error = err.to_string(), "error to create mqtt client");
                Err(MQTTError::ConnectionError {})
            }
            _ => Ok((Arc::new(client), stream)),
        }
    }

    fn default_crate_opts(&self) -> CreateOptions {
        CreateOptionsBuilder::new()
            .server_uri(&format!(
                "{}://{}:{}",
                self.transport_kind, self.mqtt_cfg.host, self.mqtt_cfg.port
            ))
            .client_id(&self.app_cfg.name)
            .finalize()
    }

    fn aws_iot_core_crate_opts(&self) -> CreateOptions {
        CreateOptionsBuilder::new()
            .server_uri(&format!(
                "ssl://{}:{}",
                self.mqtt_cfg.host, self.mqtt_cfg.port
            ))
            .client_id(&self.app_cfg.name)
            .finalize()
    }

    fn password_less_connection_opts(&self) -> ConnectOptions {
        if self.transport_kind == TransportKind::SSL {
            return ConnectOptionsBuilder::new()
                .keep_alive_interval(Duration::from_secs(60))
                .clean_session(true)
                .ssl_options(
                    SslOptionsBuilder::new()
                        .ca_path(&self.mqtt_cfg.root_ca_path)
                        .unwrap()
                        .finalize(),
                )
                .finalize();
        }

        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .clean_session(true)
            .finalize()
    }

    fn password_connection_opts(&self) -> ConnectOptions {
        if self.transport_kind == TransportKind::SSL {
            return ConnectOptionsBuilder::new()
                .keep_alive_interval(Duration::from_secs(60))
                .clean_session(true)
                .user_name(&self.mqtt_cfg.user)
                .password(&self.mqtt_cfg.password)
                .ssl_options(
                    SslOptionsBuilder::new()
                        .trust_store(&self.mqtt_cfg.root_ca_path)
                        .unwrap()
                        .verify(true)
                        .finalize(),
                )
                .finalize();
        }

        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .clean_session(true)
            .user_name(&self.mqtt_cfg.user)
            .password(&self.mqtt_cfg.password)
            .finalize()
    }

    fn aws_iot_core_connection_opts(&self) -> ConnectOptions {
        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .clean_session(false)
            .ssl_options(
                SslOptionsBuilder::new()
                    .alpn_protos(&["x-amzn-mqtt-ca"])
                    .trust_store(&self.mqtt_cfg.root_ca_path)
                    .unwrap()
                    .key_store(&self.mqtt_cfg.cert_path)
                    .unwrap()
                    .private_key(&self.mqtt_cfg.private_key_path)
                    .unwrap()
                    .ssl_version(SslVersion::Tls_1_2)
                    .verify(true)
                    .finalize(),
            )
            .finalize()
    }
}
