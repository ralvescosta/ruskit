use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SecretsManagerError {
    #[error("internal error")]
    InternalError,

    #[error("failure to send request")]
    RequestFailure,

    #[error("secret not found")]
    SecretNotFound,

    #[error("aws secret was not found")]
    AwsSecretWasNotFound,
}
