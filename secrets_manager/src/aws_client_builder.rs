use crate::AwsSecretClient;
use aws_sdk_secretsmanager as secretsmanager;
use errors::secrets_manager::SecretsManagerError;
use secretsmanager::Client;
use serde_json::Value;
use tracing::error;

#[derive(Default)]
pub struct AwsSecretClientBuilder {
    env: String,
    secret_key: String,
}

impl AwsSecretClientBuilder {
    pub fn new() -> AwsSecretClientBuilder {
        AwsSecretClientBuilder::default()
    }

    pub fn setup(mut self, env: String, secret_key: String) -> Self {
        self.env = env;
        self.secret_key = secret_key;
        self
    }

    fn secret_id(&self) -> String {
        format!("{}/{}", self.env, self.secret_key)
    }

    pub async fn build(&self) -> Result<AwsSecretClient, SecretsManagerError> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        let id = self.secret_id();

        let res = client
            .get_secret_value()
            .secret_id(&id)
            .send()
            .await
            .map_err(|e| {
                error!(
                    error = e.to_string(),
                    "failure send request to secret manager"
                );
                SecretsManagerError::RequestFailure {}
            })?;

        let Some(string) = res.secret_string() else {
            error!("secret was not found");
            return Err(SecretsManagerError::AwsSecretWasNotFound{});
        };

        let v: Value = serde_json::from_str(string).map_err(|e| {
            error!(error = e.to_string(), "error mapping secrets");
            SecretsManagerError::InternalError {}
        })?;

        Ok(AwsSecretClient { secrets: v })
    }
}
