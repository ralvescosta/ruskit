use crate::errors::MQTTError;
use configs::{AppConfigs, Configs, DynamicConfigs, MQTTConfigs};
use paho_mqtt::{
    AsyncClient, AsyncReceiver, ConnectOptions, ConnectOptionsBuilder, CreateOptions,
    CreateOptionsBuilder, Message, SslOptionsBuilder, SslVersion,
};
use std::{sync::Arc, time::Duration};
use tracing::error;

#[derive(Clone, Default)]
pub enum BrokerKind {
    #[default]
    SelfHostedWithPassword,
    SelfHostedWithoutPassword,
    AWSIoTCore,
}

pub struct MQTTClientBuilder {
    mqtt_cfg: MQTTConfigs,
    app_cfg: AppConfigs,
    broker_kind: BrokerKind,
}

impl MQTTClientBuilder {
    pub fn new() -> MQTTClientBuilder {
        MQTTClientBuilder {
            mqtt_cfg: MQTTConfigs::default(),
            app_cfg: AppConfigs::default(),
            broker_kind: BrokerKind::SelfHostedWithPassword,
        }
    }

    pub fn cfg<T>(mut self, cfgs: &Configs<T>) -> Self
    where
        T: DynamicConfigs,
    {
        self.mqtt_cfg = cfgs.mqtt.clone();
        self.app_cfg = self.app_cfg.clone();
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

    pub async fn build(
        self,
    ) -> Result<(Arc<AsyncClient>, AsyncReceiver<Option<Message>>), MQTTError> {
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

    fn crate_opts_self_hosted(&self) -> CreateOptions {
        CreateOptionsBuilder::new()
            .server_uri(&format!(
                "tcp://{}:{}",
                self.mqtt_cfg.host, self.mqtt_cfg.port
            ))
            .client_id(&self.app_cfg.name)
            .finalize()
    }

    fn crate_opts_aws_iot_core(&self) -> CreateOptions {
        CreateOptionsBuilder::new()
            .server_uri(&format!(
                "ssl://{}:{}",
                self.mqtt_cfg.host, self.mqtt_cfg.port
            ))
            .client_id(&self.app_cfg.name)
            .finalize()
    }

    fn conn_opts_for_self_hosted_without_password(&self) -> ConnectOptions {
        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .clean_session(true)
            .finalize()
    }

    fn conn_opts_for_self_hosted_with_password(&self) -> ConnectOptions {
        ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(60))
            .clean_session(true)
            .user_name(&self.mqtt_cfg.user)
            .password(&self.mqtt_cfg.password)
            .finalize()
    }

    fn conn_opts_aws_iot_core(&self) -> ConnectOptions {
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
