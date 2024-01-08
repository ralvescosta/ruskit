#[derive(Debug, Clone, Default)]
pub enum SecretsManagerKind {
    #[default]
    None,
    AWSSecretManager,
}

impl From<&str> for SecretsManagerKind {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "AWS" => SecretsManagerKind::AWSSecretManager,
            _ => SecretsManagerKind::None,
        }
    }
}

impl From<&String> for SecretsManagerKind {
    fn from(value: &String) -> Self {
        match value.to_uppercase().as_str() {
            "AWS" => SecretsManagerKind::AWSSecretManager,
            _ => SecretsManagerKind::None,
        }
    }
}
