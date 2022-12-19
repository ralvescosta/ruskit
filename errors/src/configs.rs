use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConfigsError {
    #[error("internal error")]
    InternalError,

    #[error("error to load secrets from secret manager - `{0}`")]
    SecretLoadingError(String)
}
