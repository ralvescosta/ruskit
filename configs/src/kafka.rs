#[derive(Debug, Clone)]
pub struct KafkaConfigs {
    pub host: String,
    pub port: u64,
    pub timeout: u64,
    pub security_protocol: String,
    pub sasl_mechanisms: String,
    pub user: String,
    pub password: String,
}

impl Default for KafkaConfigs {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 9094,
            timeout: 6000,
            security_protocol: "SASL_SSL".into(),
            sasl_mechanisms: "PLAIN".into(),
            user: Default::default(),
            password: Default::default(),
        }
    }
}
