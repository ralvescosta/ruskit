use crate::{errors::SecretsManagerError, SecretClient};
use serde_json::Value;
use tracing::error;

pub struct AwsSecretClient {
    pub(crate) secrets: Value,
}

impl SecretClient for AwsSecretClient {
    fn get_by_key(&self, key: &str) -> Result<String, SecretsManagerError> {
        let key = key.strip_prefix("!").unwrap_or_default();
        let value = self.secrets[key].clone();

        let Value::String(secret) = value else {
            error!(key = key, "secret {} was not found", key);
            return Err(SecretsManagerError::SecretNotFound{});
        };

        Ok(secret)
    }
}
