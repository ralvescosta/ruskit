use crate::{errors::MigrationError, service::MigratorDriver};
use async_trait::async_trait;
use deadpool_postgres::{tokio_postgres::error::SqlState, Object, Pool};
use std::{fs, sync::Arc};
use tracing::{debug, error};

pub struct PostgresDriver {
    pool: Arc<Pool>,
}

impl PostgresDriver {
    pub fn new(pool: Arc<Pool>) -> PostgresDriver {
        PostgresDriver { pool }
    }
}

#[async_trait]
impl MigratorDriver for PostgresDriver {
    async fn migration_table(&self) -> Result<(), MigrationError> {
        let conn = self.get_conn().await?;

        self.begin(&conn).await?;

        let query = "SELECT migrate FROM migrations LIMIT 1";
        let Err(err) = conn.prepare(query).await else {
            debug!("migration table already created");
            self.commit(&conn).await?;
            return Ok(());
        };

        if err.code().unwrap_or(&SqlState::SYNTAX_ERROR) != &SqlState::INVALID_TABLE_DEFINITION {
            error!(error = err.to_string(), "unexpected error");
            self.rollback(&conn).await?;
            return Err(MigrationError::InternalError {});
        }

        let query = "
            CREATE TABLE migrations (
                id serial NOT NULL,
                migrate TEXT,
                executed_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
                rollback_at timestamptz
            )";

        let Ok(statement) = conn.prepare(query).await else {
            error!("error to prepare create migrations table query");
            self.rollback(&conn).await?;
            return Err(MigrationError::CreateMigrationsTableErr {});
        };

        let Ok(_) = conn.query(&statement, &[]).await else {
            error!("error to execute create migrations table query");
            self.rollback(&conn).await?;
            return Err(MigrationError::CreateMigrationsTableErr {});
        };

        self.commit(&conn).await?;

        Ok(())
    }

    async fn up(
        &self,
        path: Option<&String>,
        _migration: Option<&String>,
    ) -> Result<(), MigrationError> {
        let mut migrations_path = "./migrations/sql/";

        if path.is_some() {
            migrations_path = path.unwrap().as_str();
        }

        let conn = self.get_conn().await?;

        self.begin(&conn).await?;

        for res_dir_entry in fs::read_dir(migrations_path).unwrap() {
            let dir_entry = res_dir_entry.unwrap();
            let file_name = dir_entry.file_name().into_string().unwrap_or_default();

            if !file_name.contains("up") || !file_name.contains(".sql") {
                continue;
            }

            if self.migrate_executed_already(&conn, &file_name).await? {
                continue;
            }

            let query = fs::read_to_string(dir_entry.path()).map_err(|e| {
                error!(error = e.to_string(), "error to read migration file");
                MigrationError::InternalError {}
            })?;

            let statement = match conn.prepare(&query).await {
                Err(err) => {
                    error!(
                        error = err.to_string(),
                        migrate = file_name,
                        "error to execute prepare migrate query"
                    );
                    self.rollback(&conn).await?;
                    Err(MigrationError::PrepareStatementErr {})
                }
                Ok(s) => Ok(s),
            }?;

            match conn.query(&statement, &[]).await {
                Err(err) => {
                    error!(error = err.to_string(), "error to execute migrate query");
                    self.rollback(&conn).await?;
                    Err(MigrationError::MigrateQueryErr {})
                }
                _ => Ok(()),
            }?;

            let statement = match conn
                .prepare("INSERT INTO migrations (migrate) values (?)")
                .await
            {
                Err(err) => {
                    error!(
                        error = err.to_string(),
                        migrate = file_name,
                        "error to execute prepare insert in migrations table"
                    );
                    self.rollback(&conn).await?;
                    Err(MigrationError::PrepareStatementErr {})
                }
                Ok(s) => Ok(s),
            }?;

            match conn.query(&statement, &[&file_name]).await {
                Err(err) => {
                    error!(
                        error = err.to_string(),
                        "error to inset migrate in migrations table"
                    );
                    self.rollback(&conn).await?;
                    Err(MigrationError::InsertErr {})
                }
                _ => Ok(()),
            }?;
        }

        Ok(())
    }

    async fn down(
        &self,
        _path: Option<&String>,
        _migration: Option<&String>,
    ) -> Result<(), MigrationError> {
        Ok(())
    }
}

impl PostgresDriver {
    async fn get_conn(&self) -> Result<Object, MigrationError> {
        match self.pool.get().await {
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error to retrieve a db connection from pool"
                );
                Err(MigrationError::DbConnectionErr {})
            }
            Ok(p) => Ok(p),
        }
    }

    async fn begin(&self, conn: &Object) -> Result<(), MigrationError> {
        match conn.query("begin transaction migration;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to begin transaction");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    async fn commit(&self, conn: &Object) -> Result<(), MigrationError> {
        match conn.query("commit migration;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to commit transaction");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    async fn rollback(&self, conn: &Object) -> Result<(), MigrationError> {
        match conn.query("rollback migration;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to rollback transaction");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    async fn migrate_executed_already(
        &self,
        conn: &Object,
        file_name: &String,
    ) -> Result<bool, MigrationError> {
        let query = "SELECT migrate FROM migrations WHERE migrate = ?";
        let statement = match conn.prepare(query).await {
            Err(err) => {
                error!(error = err.to_string(), "error to rollback transaction");
                Err(MigrationError::InternalError {})
            }
            Ok(s) => Ok(s),
        }?;

        match conn.query_one(&statement, &[file_name]).await {
            Err(err) => {
                if err.code().unwrap_or(&SqlState::NO_DATA_FOUND) != &SqlState::NO_DATA_FOUND {
                    error!(error = err.to_string(), "unsuspected error");
                    return Err(MigrationError::InternalError {});
                }
                Ok(false)
            }
            _ => Ok(true),
        }
    }
}
