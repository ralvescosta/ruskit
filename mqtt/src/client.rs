use crate::errors::MQTTError;
use configs::{Configs, DynamicConfigs, MQTTBrokerKind, MQTTConfigs};
use paho_mqtt::{
    AsyncClient, AsyncReceiver, ConnectOptions, ConnectOptionsBuilder, CreateOptions,
    CreateOptionsBuilder, Message, SslOptions, SslOptionsBuilder, SslVersion,
};
use std::{sync::Arc, time::Duration};
use tracing::error;

#[derive(Clone, Default)]
pub enum ConnectionKind {
    #[default]
    WithPassword,
    WithoutPassword,
    AWSIoTCore,
}

pub struct MQTTClient {
    crate_opts: CreateOptions,
    connection_opts: ConnectOptions,
}

impl MQTTClient {
    pub fn new<T>(cfgs: &Configs<T>) -> MQTTClient
    where
        T: DynamicConfigs,
    {
        let crate_opts = default_crate_opts(cfgs);

        let connection_opts = match cfgs.mqtt.broker_kind {
            MQTTBrokerKind::AWSIoTCore => aws_iot_core_connection_opts(&cfgs.mqtt),
            MQTTBrokerKind::Default => password_connection_opts(&cfgs.mqtt),
        };

        return MQTTClient {
            crate_opts,
            connection_opts,
        };
    }

    pub async fn connect(
        self,
    ) -> Result<(Arc<AsyncClient>, AsyncReceiver<Option<Message>>), MQTTError> {
        let mut client = match AsyncClient::new(self.crate_opts) {
            Err(err) => {
                error!(error = err.to_string(), "error to create mqtt client");
                Err(MQTTError::ConnectionError {})
            }
            Ok(c) => Ok(c),
        }?;

        let stream = client.get_stream(2048);

        match client.connect(self.connection_opts.clone()).await {
            Err(err) => {
                error!(error = err.to_string(), "error to create mqtt client");
                Err(MQTTError::ConnectionError {})
            }
            _ => Ok((Arc::new(client), stream)),
        }
    }
}

fn default_crate_opts<T>(cfgs: &Configs<T>) -> CreateOptions
where
    T: DynamicConfigs,
{
    CreateOptionsBuilder::new()
        .server_uri(&format!(
            "{}://{}:{}",
            cfgs.mqtt.transport, cfgs.mqtt.host, cfgs.mqtt.port
        ))
        .client_id(&cfgs.app.name)
        .finalize()
}

fn password_connection_opts(cfgs: &MQTTConfigs) -> ConnectOptions {
    let mut ssl_options = SslOptions::default();

    if !cfgs.root_ca_path.is_empty() {
        ssl_options = SslOptionsBuilder::new()
            .trust_store(&cfgs.root_ca_path)
            .unwrap()
            .verify(true)
            .finalize();
    }

    ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(60))
        .clean_session(true)
        .user_name(&cfgs.user)
        .password(&cfgs.password)
        .ssl_options(ssl_options)
        .finalize()
}

fn aws_iot_core_connection_opts(cfgs: &MQTTConfigs) -> ConnectOptions {
    let mut ssl_options = SslOptions::default();

    if !cfgs.root_ca_path.is_empty()
        && !cfgs.cert_path.is_empty()
        && !cfgs.private_key_path.is_empty()
    {
        ssl_options = SslOptionsBuilder::new()
            .alpn_protos(&["x-amzn-mqtt-ca"])
            .ssl_version(SslVersion::Tls_1_2)
            .verify(true)
            .trust_store(&cfgs.root_ca_path)
            .unwrap()
            .key_store(&cfgs.cert_path)
            .unwrap()
            .private_key(&cfgs.private_key_path)
            .unwrap()
            .finalize();
    }

    if cfgs.root_ca_path.is_empty()
        && !cfgs.cert_path.is_empty()
        && !cfgs.private_key_path.is_empty()
    {
        ssl_options = SslOptionsBuilder::new()
            .alpn_protos(&["x-amzn-mqtt-ca"])
            .ssl_version(SslVersion::Tls_1_2)
            .verify(true)
            .key_store(&cfgs.cert_path)
            .unwrap()
            .private_key(&cfgs.private_key_path)
            .unwrap()
            .finalize();
    }

    if cfgs.root_ca_path.is_empty()
        && cfgs.cert_path.is_empty()
        && !cfgs.private_key_path.is_empty()
    {
        ssl_options = SslOptionsBuilder::new()
            .alpn_protos(&["x-amzn-mqtt-ca"])
            .ssl_version(SslVersion::Tls_1_2)
            .verify(true)
            .private_key(&cfgs.private_key_path)
            .unwrap()
            .finalize();
    }

    if cfgs.root_ca_path.is_empty()
        && !cfgs.cert_path.is_empty()
        && cfgs.private_key_path.is_empty()
    {
        ssl_options = SslOptionsBuilder::new()
            .alpn_protos(&["x-amzn-mqtt-ca"])
            .ssl_version(SslVersion::Tls_1_2)
            .verify(true)
            .key_store(&cfgs.cert_path)
            .unwrap()
            .finalize();
    }

    ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(60))
        .clean_session(false)
        .ssl_options(ssl_options)
        .finalize()
}
