use crate::{types::BrokerKind, MqttClientImpl};
use env::Config;
use errors::mqtt::MqttError;
use paho_mqtt::{
    AsyncClient, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder,
    SslOptionsBuilder, SslVersion, MQTT_VERSION_3_1_1,
};
use std::{sync::Arc, time::Duration};
use tracing::error;

pub struct MqttClientBuilder {
    cfg: Config,
    broker_kind: BrokerKind,
}

impl MqttClientBuilder {
    pub fn new() -> MqttClientBuilder {
        MqttClientBuilder {
            cfg: Config::default(),
            broker_kind: BrokerKind::SelfHostedWithPassword,
        }
    }

    pub fn cfg(mut self, cfg: Config) -> Self {
        self.cfg = cfg;
        return self;
    }

    pub fn aws(mut self) -> Self {
        self.broker_kind = BrokerKind::AWSIoTCore;
        return self;
    }

    pub fn self_hosted_authenticated(mut self) -> Self {
        self.broker_kind = BrokerKind::SelfHostedWithPassword;
        return self;
    }

    pub fn self_hosted(mut self) -> Self {
        self.broker_kind = BrokerKind::SelfHostedWithoutPassword;
        return self;
    }

    pub async fn build(self) -> Result<MqttClientImpl, MqttError> {
        let crate_opts = match self.broker_kind {
            BrokerKind::AWSIoTCore => self.crate_opts_aws_iot_core(),
            _ => self.crate_opts_self_hosted(),
        };

        let conn_opts = match self.broker_kind {
            BrokerKind::SelfHostedWithPassword => self.conn_opts_for_self_hosted_with_password(),
            BrokerKind::SelfHostedWithoutPassword => {
                self.conn_opts_for_self_hosted_without_password()
            }
            BrokerKind::AWSIoTCore => self.conn_opts_aws_iot_core(),
        };

        let mut client = AsyncClient::new(crate_opts).map_err(|e| {
            error!(error = e.to_string(), "error to create mqtt client");
            MqttError::ConnectionError {}
        })?;

        let stream = client.get_stream(2048);

        client.connect(conn_opts.clone()).await.map_err(|e| {
            error!(error = e.to_string(), "error to create mqtt client");
            MqttError::ConnectionError {}
        })?;

        return Ok(MqttClientImpl {
            client: Arc::new(client),
            stream,
        });
    }

    fn crate_opts_self_hosted(&self) -> CreateOptions {
        CreateOptionsBuilder::new()
            .server_uri(&format!(
                "tcp://{}:{}",
                self.cfg.mqtt.host, self.cfg.mqtt.port
            ))
            .client_id(&self.cfg.app.name)
            .finalize()
    }

    fn crate_opts_aws_iot_core(&self) -> CreateOptions {
        CreateOptionsBuilder::new()
            .server_uri(&format!(
                "ssl://{}:{}",
                self.cfg.mqtt.host, self.cfg.mqtt.port
            ))
            .client_id(&self.cfg.app.name)
            .finalize()
    }

    fn conn_opts_for_self_hosted_without_password(&self) -> ConnectOptions {
        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .mqtt_version(MQTT_VERSION_3_1_1)
            .clean_session(true)
            .finalize()
    }

    fn conn_opts_for_self_hosted_with_password(&self) -> ConnectOptions {
        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .mqtt_version(MQTT_VERSION_3_1_1)
            .clean_session(true)
            .user_name(&self.cfg.mqtt.user)
            .password(&self.cfg.mqtt.password)
            .finalize()
    }

    fn conn_opts_aws_iot_core(&self) -> ConnectOptions {
        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .mqtt_version(MQTT_VERSION_3_1_1)
            .clean_session(false)
            .ssl_options(
                SslOptionsBuilder::new()
                    .alpn_protos(&["x-amzn-mqtt-ca"])
                    .trust_store(&self.cfg.mqtt.root_ca_path)
                    .unwrap()
                    .key_store(&self.cfg.mqtt.cert_path)
                    .unwrap()
                    .private_key(&self.cfg.mqtt.private_key_path)
                    .unwrap()
                    .ssl_version(SslVersion::Tls_1_2)
                    .verify(true)
                    .finalize(),
            )
            .finalize()
    }
}
