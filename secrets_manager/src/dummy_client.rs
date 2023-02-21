use crate::{errors::SecretsManagerError, SecretClient};

pub struct DummyClient;

impl SecretClient for DummyClient {
    fn get_by_key(&self, _key: &str) -> Result<String, SecretsManagerError> {
        Ok(String::new())
    }
}

impl DummyClient {
    pub fn new() -> DummyClient {
        DummyClient {}
    }
}
