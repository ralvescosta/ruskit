use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MigrationError {
    #[error("internal error")]
    InternalError,

    #[error("db connection error")]
    DbConnectionErr,

    #[error("prepare statement error")]
    PrepareStatementErr,

    #[error("migrate query error")]
    MigrateQueryErr,

    #[error("create table error")]
    CreateMigrationsTableErr,

    #[error("select error")]
    SelectErr,

    #[error("insert error")]
    InsertErr,

    #[error("update error")]
    UpdateErr,

    #[error("invalid argument: `{0}`")]
    InvalidArgumentErr(String),
}
