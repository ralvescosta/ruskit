use errors::secrets_manager::SecretsManagerError;

pub trait SecretClient {
    fn get_by_key(&self, key: &str) -> Result<String, SecretsManagerError>;
}
