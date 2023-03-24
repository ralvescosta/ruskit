use crate::{errors::SecretsManagerError, AWSSecretClient};
use aws_sdk_secretsmanager as secretsmanager;
#[cfg(test)]
use mockall::*;
#[cfg(mock)]
use mockall::*;
use secretsmanager::Client;
use serde_json::Value;
use tracing::error;

#[derive(Default)]
pub struct AWSSecretClientBuilder {
    env: String,
    secret_key: String,
}

#[cfg_attr(test, automock)]
#[cfg_attr(mock, automock)]
impl AWSSecretClientBuilder {
    pub fn new() -> AWSSecretClientBuilder {
        AWSSecretClientBuilder::default()
    }

    pub fn setup(mut self, env: String, secret_key: String) -> Self {
        self.env = env;
        self.secret_key = secret_key;
        self
    }

    fn secret_id(&self) -> String {
        format!("{}/{}", self.env, self.secret_key)
    }

    pub async fn build(&self) -> Result<AWSSecretClient, SecretsManagerError> {
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

        Ok(AWSSecretClient { secrets: v })
    }
}
