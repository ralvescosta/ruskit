use crate::errors::SecretsManagerError;
use async_trait::async_trait;
#[cfg(test)]
use mockall::*;
#[cfg(mock)]
use mockall::*;

#[cfg_attr(test, automock)]
#[cfg_attr(mock, automock)]
#[async_trait]
pub trait SecretClient {
    fn get_by_key(&self, key: &str) -> Result<String, SecretsManagerError>;
}
