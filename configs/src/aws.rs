#[derive(Debug, Clone)]
pub struct AwsConfigs {
    ///Default: local
    pub access_key_id: Option<String>,
    ///Default: local
    pub secret_access_key: Option<String>,
    ///Default:
    pub session_token: Option<String>,
}

impl Default for AwsConfigs {
    fn default() -> Self {
        Self {
            access_key_id: Some("local".to_owned()),
            secret_access_key: Some("local".to_owned()),
            session_token: Default::default(),
        }
    }
}
