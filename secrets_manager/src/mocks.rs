use crate::{errors::SecretsManagerError, SecretClient};
use async_trait::async_trait;
use mockall::*;

mock! {
  pub SecretClientImpl {}

  #[async_trait]
  impl SecretClient for SecretClientImpl {
    fn get_by_key(&self, key: &str) -> Result<String, SecretsManagerError> {
      todo!()
    }

  }
}
