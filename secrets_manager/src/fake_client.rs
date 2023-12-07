use crate::{errors::SecretsManagerError, SecretClient};

#[derive(Default)]
pub struct FakeSecretClient;

impl SecretClient for FakeSecretClient {
    fn get_by_key(&self, _key: &str) -> Result<String, SecretsManagerError> {
        Ok(String::new())
    }
}

impl FakeSecretClient {
    pub fn new() -> FakeSecretClient {
        FakeSecretClient {}
    }
}
