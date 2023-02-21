use crate::errors::SecretsManagerError;
use async_trait::async_trait;

#[async_trait]
pub trait SecretClient {
    fn get_by_key(&self, key: &str) -> Result<String, SecretsManagerError>;
}
