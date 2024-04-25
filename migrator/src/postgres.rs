use crate::{errors::MigrationError, service::MigratorDriver};
use async_trait::async_trait;
use deadpool_postgres::{tokio_postgres::error::SqlState, Object, Pool};
use std::{fs, path::PathBuf, sync::Arc};
use tracing::{debug, error, warn};

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

        if err.code().unwrap_or(&SqlState::SYNTAX_ERROR) != &SqlState::UNDEFINED_TABLE {
            error!(error = err.to_string(), "unexpected error");
            self.rollback(&conn).await?;
            return Err(MigrationError::InternalError {});
        }

        //if there is no table, the last query will failure
        //because of that we need to commit the tx and create a new one.
        self.commit(&conn).await?;
        self.begin(&conn).await?;

        let query = "
            CREATE TABLE migrations (
                id serial NOT NULL,
                migrate TEXT,
                executed_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
                rollback_at timestamptz
            )
        ";

        let statement = match conn.prepare(query).await {
            Ok(s) => Ok(s),
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error to prepare create migrations table query"
                );
                self.rollback(&conn).await?;
                return Err(MigrationError::CreateMigrationsTableErr {});
            }
        }?;

        match conn.query(&statement, &[]).await {
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error to execute create migrations table query"
                );
                self.rollback(&conn).await?;
                return Err(MigrationError::CreateMigrationsTableErr {});
            }
            Ok(_) => {
                self.commit(&conn).await?;
                Ok(())
            }
        }
    }

    async fn up(&self, path: Option<&str>, _migration: Option<&str>) -> Result<(), MigrationError> {
        let mut migrations_path = "./bins/migrations/sql/";

        if path.is_some() {
            migrations_path = path.unwrap();
        }

        let conn = self.get_conn().await?;

        let dirs = self.read_dir_sorted_for_up(migrations_path)?;

        self.begin(&conn).await?;

        for dir in dirs {
            let (file_name, file_path) = dir;

            if self.migrate_executed_already(&conn, &file_name).await? {
                continue;
            }

            let query = match fs::read_to_string(file_path) {
                Ok(q) => Ok(q),
                Err(err) => {
                    error!(error = err.to_string(), "error to read migration file");
                    self.rollback(&conn).await?;
                    Err(MigrationError::InternalError {})
                }
            }?;

            match conn.batch_execute(&query).await {
                Err(err) => {
                    if err.code().unwrap_or(&SqlState::SYNTAX_ERROR) == &SqlState::DUPLICATE_TABLE {
                        warn!(
                            error = err.to_string(),
                            "there is a table with the same name"
                        );
                        return Ok(());
                    }

                    error!(error = err.to_string(), "error to execute migrate query");
                    self.rollback(&conn).await?;
                    Err(MigrationError::MigrateQueryErr {})
                }
                _ => Ok(()),
            }?;

            let statement = match conn
                .prepare("INSERT INTO migrations (migrate) values ($1)")
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

        self.commit(&conn).await?;

        Ok(())
    }

    async fn down(
        &self,
        path: Option<&str>,
        _migration: Option<&str>,
    ) -> Result<(), MigrationError> {
        let mut migrations_path = "./bins/migrations/sql/";

        if path.is_some() {
            migrations_path = path.unwrap();
        }

        let conn = self.get_conn().await?;

        let dirs = self.read_dir_sorted_for_down(migrations_path)?;

        self.begin(&conn).await?;

        for dir in dirs {
            let (file_name, file_path) = dir;

            if self.migrate_executed_already(&conn, &file_name).await? {
                continue;
            }

            let query = match fs::read_to_string(file_path) {
                Ok(q) => Ok(q),
                Err(err) => {
                    error!(error = err.to_string(), "error to read migration file");
                    self.rollback(&conn).await?;
                    Err(MigrationError::InternalError {})
                }
            }?;

            match conn.batch_execute(&query).await {
                Err(err) => {
                    if err.code().unwrap_or(&SqlState::SYNTAX_ERROR) == &SqlState::DUPLICATE_TABLE {
                        warn!(
                            error = err.to_string(),
                            "there is a table with the same name"
                        );
                        return Ok(());
                    }

                    error!(error = err.to_string(), "error to execute migrate query");
                    self.rollback(&conn).await?;
                    Err(MigrationError::MigrateQueryErr {})
                }
                _ => Ok(()),
            }?;

            let statement = match conn
                .prepare("INSERT INTO migrations (migrate) values ($1)")
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

        self.commit(&conn).await?;

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
        match conn.query("begin transaction;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to begin transaction");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    async fn commit(&self, conn: &Object) -> Result<(), MigrationError> {
        match conn.query("commit transaction;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to commit transaction");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    async fn rollback(&self, conn: &Object) -> Result<(), MigrationError> {
        match conn.query("rollback transaction;", &[]).await {
            Err(err) => {
                error!(error = err.to_string(), "error to rollback transaction");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    fn read_dir_sorted_for_up(&self, path: &str) -> Result<Vec<(String, PathBuf)>, MigrationError> {
        let dirs = match fs::read_dir(path) {
            Ok(d) => Ok(d),
            Err(err) => {
                error!(error = err.to_string(), "error reading the migration path");
                Err(MigrationError::InternalError {})
            }
        }?;

        let mut migrations: Vec<(String, PathBuf)> = dirs
            .map(|d| {
                let dir = d.unwrap();
                (
                    dir.file_name().into_string().unwrap_or_default(),
                    dir.path(),
                )
            })
            .filter(|f| f.0.contains("up") && f.0.contains(".sql"))
            .collect();

        migrations.sort_by_key(|v| v.0.clone());

        Ok(migrations)
    }

    fn read_dir_sorted_for_down(
        &self,
        path: &str,
    ) -> Result<Vec<(String, PathBuf)>, MigrationError> {
        let dirs = match fs::read_dir(path) {
            Ok(d) => Ok(d),
            Err(err) => {
                error!(error = err.to_string(), "error reading the migration path");
                Err(MigrationError::InternalError {})
            }
        }?;

        let mut migrations: Vec<(String, PathBuf)> = dirs
            .map(|d| {
                let dir = d.unwrap();
                (
                    dir.file_name().into_string().unwrap_or_default(),
                    dir.path(),
                )
            })
            .filter(|f| f.0.contains("down") && f.0.contains(".sql"))
            .collect();

        migrations.sort_by_key(|v| v.0.clone());

        Ok(migrations)
    }

    async fn migrate_executed_already(
        &self,
        conn: &Object,
        file_name: &String,
    ) -> Result<bool, MigrationError> {
        let query = "SELECT migrate FROM migrations WHERE migrate = $1";
        let statement = match conn.prepare(query).await {
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error prepare select migration query"
                );
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
