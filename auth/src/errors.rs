use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AuthError {
    #[error("internal error")]
    InternalError,

    #[error("could not retrieve JWKS")]
    CouldNotRetrieveJWKS,

    #[error("error to load secrets from secret manager - `{0}`")]
    InvalidToken(String),

    #[error("failed to deserialize token")]
    FailedToDeserializeToken,

    #[error("failed to retrieve claim")]
    FailedToRetrieveClaim,

    #[error("failed to retrieve user custom data claim")]
    FailedToRetrieveUserCustomDataClaim,

    #[error("failed to retrieve scope claim")]
    FailedToRetrieveScopeClaim,
}
