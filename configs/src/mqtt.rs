use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub enum MQTTBrokerKind {
    #[default]
    Default,
    AWSIoTCore,
}

impl From<&str> for MQTTBrokerKind {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "AWSIoTCore" => MQTTBrokerKind::AWSIoTCore,
            _ => MQTTBrokerKind::Default,
        }
    }
}

impl From<&String> for MQTTBrokerKind {
    fn from(value: &String) -> Self {
        match value.to_uppercase().as_str() {
            "AWSIoTCore" => MQTTBrokerKind::AWSIoTCore,
            _ => MQTTBrokerKind::Default,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum MQTTTransport {
    #[default]
    TCP,
    SSL,
    WS,
}

impl From<&str> for MQTTTransport {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "SSL" => MQTTTransport::SSL,
            "WS" => MQTTTransport::WS,
            _ => MQTTTransport::TCP,
        }
    }
}

impl From<&String> for MQTTTransport {
    fn from(value: &String) -> Self {
        match value.to_uppercase().as_str() {
            "SSL" => MQTTTransport::SSL,
            "WS" => MQTTTransport::WS,
            _ => MQTTTransport::TCP,
        }
    }
}

impl Display for MQTTTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MQTTTransport::TCP => write!(f, "tcp"),
            MQTTTransport::SSL => write!(f, "ssl"),
            MQTTTransport::WS => write!(f, "ws"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MQTTConfigs {
    pub broker_kind: MQTTBrokerKind,
    ///Default: localhost
    pub host: String,
    //Default: tcp
    pub transport: MQTTTransport,
    ///Default: 1883
    pub port: u64,
    ///Default: mqtt_user
    pub user: String,
    /// Default: password
    pub password: String,
    ///Used with Public Cloud Brokers
    pub device_name: String,
    ///Used with Public Cloud Brokers
    pub root_ca_path: String,
    ///Used with Public Cloud Brokers
    pub cert_path: String,
    ///Used with Public Cloud Brokers
    pub private_key_path: String,
}

impl Default for MQTTConfigs {
    fn default() -> Self {
        Self {
            broker_kind: MQTTBrokerKind::default(),
            host: "localhost".to_owned(),
            transport: MQTTTransport::default(),
            port: 1883,
            user: "mqtt".to_owned(),
            password: "password".to_owned(),
            device_name: Default::default(),
            root_ca_path: Default::default(),
            cert_path: Default::default(),
            private_key_path: Default::default(),
        }
    }
}
