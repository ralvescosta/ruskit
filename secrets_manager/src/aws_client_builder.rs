use crate::{errors::SecretsManagerError, AWSSecretClient};
use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager as secretsmanager;
#[cfg(test)]
use mockall::*;
#[cfg(mock)]
use mockall::*;
use secretsmanager::Client;
use tracing::error;

#[derive(Default)]
pub struct AWSSecretClientBuilder {
    env: String,
    secret_key: String,
}

#[cfg_attr(test, automock)]
#[cfg_attr(mock, automock)]
impl AWSSecretClientBuilder {
    pub fn new(env: String, secret_key: String) -> AWSSecretClientBuilder {
        AWSSecretClientBuilder { env, secret_key }
    }

    fn secret_id(&self) -> String {
        format!("{}/{}", self.env, self.secret_key)
    }

    pub async fn build(&self) -> Result<AWSSecretClient, SecretsManagerError> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        let id = self.secret_id();

        let output = match client.get_secret_value().secret_id(&id).send().await {
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "failure send request to secret manager"
                );
                Err(SecretsManagerError::RequestFailure {})
            }
            Ok(s) => Ok(s),
        }?;

        let Some(string) = output.secret_string() else {
            error!("secret was not found");
            return Err(SecretsManagerError::AwsSecretWasNotFound {});
        };

        match serde_json::from_str(string) {
            Err(err) => {
                error!(error = err.to_string(), "error mapping secrets");
                Err(SecretsManagerError::InternalError {})
            }
            Ok(v) => Ok(AWSSecretClient { secrets: v }),
        }
    }
}
