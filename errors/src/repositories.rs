use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RepositoriesError {
    #[error("internal error")]
    InternalError,
}
