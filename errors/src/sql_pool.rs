use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SqlPoolError {
    #[error("internal error")]
    InternalError,

    #[error("postgres connection error `{0}`")]
    PostgresConnectionErr(String),

    #[error("sqlite connection error `{0}`")]
    SqliteConnectionErr(String),
}
