use aws_sdk_secretsmanager as secretsmanager;
use errors::secrets_manager::SecretsManagerError;
use secretsmanager::Client;
use serde_json::Value;
use tracing::error;

use crate::AwsSecretClient;

pub struct AwsSecretClientBuilder {
    env: String,
    app_ctx: String,
}

impl AwsSecretClientBuilder {
    pub fn new() -> AwsSecretClientBuilder {
        AwsSecretClientBuilder {
            env: String::new(),
            app_ctx: String::new(),
        }
    }

    pub fn setup(mut self, env: String, app_ctx: String) -> Self {
        self.env = env;
        self.app_ctx = app_ctx;
        self
    }

    fn secret_id(&self) -> String {
        format!("{}/{}", self.env, self.app_ctx)
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
                error!(error = e.to_string(), "failure to get key from aws");
                SecretsManagerError::AwsSecretWasNotFound {}
            })?;

        let Some(string) = res.secret_string() else {
            error!("secret was not found");
            return Err(SecretsManagerError::InternalError{});
        };

        let v: Value = serde_json::from_str(string).map_err(|e| {
            error!(error = e.to_string(), "error mapping secrets");
            SecretsManagerError::InternalError {}
        })?;

        Ok(AwsSecretClient { secrets: v })
    }
}
