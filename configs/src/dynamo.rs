#[derive(Debug, Clone)]
pub struct DynamoConfigs {
    ///Default: localhost
    pub endpoint: String,
    ///Default: us-east-1
    pub region: String,
    ///Default: table
    pub table: String,
    ///Default: 31536000
    pub expire: u64,
}

impl Default for DynamoConfigs {
    fn default() -> Self {
        Self {
            endpoint: "localhost".to_owned(),
            region: "us-east-1".to_owned(),
            table: Default::default(),
            expire: 31536000,
        }
    }
}
