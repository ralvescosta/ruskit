use crate::{errors::SecretsManagerError, SecretClient};
#[cfg(test)]
use mockall::*;
#[cfg(mock)]
use mockall::*;
use serde_json::Value;
use tracing::error;

#[derive(Default)]
pub struct AWSSecretClient {
    pub(crate) secrets: Value,
}

#[cfg_attr(test, automock)]
#[cfg_attr(mock, automock)]
impl SecretClient for AWSSecretClient {
    fn get_by_key(&self, key: &str) -> Result<String, SecretsManagerError> {
        let key = key.strip_prefix('!').unwrap_or_default();
        let value = self.secrets[key].clone();

        let Value::String(secret) = value else {
            error!(key = key, "secret {} was not found", key);
            return Err(SecretsManagerError::SecretNotFound {});
        };

        Ok(secret)
    }
}
