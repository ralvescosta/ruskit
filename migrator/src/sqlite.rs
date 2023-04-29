use crate::{errors::MigrationError, service::MigratorDriver};
use async_trait::async_trait;
use deadpool_sqlite::{rusqlite::ErrorCode, Object, Pool};
use std::{fs, sync::Arc};
use tracing::{debug, error, warn};

pub struct SqliteDriver {
    pool: Arc<Pool>,
}

impl SqliteDriver {
    pub fn new(pool: Arc<Pool>) -> SqliteDriver {
        SqliteDriver { pool }
    }
}

#[async_trait]
impl MigratorDriver for SqliteDriver {
    async fn migration_table(&self) -> Result<(), MigrationError> {
        let conn = self.get_conn().await?;

        self.begin(&conn).await?;

        match conn
            .interact(|conn| {
                let query = "SELECT migrate FROM migrations limit 1";
                let Err(err) = conn.prepare(query) else {
                debug!("migration table already created");
                return Ok(());
            };

                if err.sqlite_error_code().unwrap_or(ErrorCode::NotFound) != ErrorCode::Unknown {
                    error!(error = err.to_string(), "unexpected error");
                    return Err(MigrationError::InternalError {});
                }

                let query = "
            CREATE TABLE migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                migrate TEXT,
                executed_at TIMESTAMPZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
                rollback_at TIMESTAMPZ
            )";

                let Ok(mut statement) = conn.prepare(query) else {
                error!("error to prepare create migrations table query");
                return Err(MigrationError::CreateMigrationsTableErr {});
            };

                let Ok(_) = statement.execute([]) else {
                error!("error to execute create migrations table query");
                return Err(MigrationError::CreateMigrationsTableErr {});
            };

                Ok(())
            })
            .await
        {
            Err(_) => {
                self.rollback(&conn).await?;
                return Err(MigrationError::InternalError {});
            }
            _ => {
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

        self.begin(&conn).await?;

        for res_dir_entry in fs::read_dir(migrations_path).unwrap() {
            let dir_entry = res_dir_entry.unwrap();

            let file_name = dir_entry.file_name().into_string().unwrap_or_default();

            if !file_name.contains("up") || !file_name.contains(".sql") {
                warn!(file = file_name, "skipping migrate");
                continue;
            }

            if self.migrate_executed_already(&conn, &file_name).await? {
                continue;
            }

            let query: String = match fs::read_to_string(dir_entry.path()) {
                Err(err) => {
                     error!(error = e.to_string(), "error to read migration file");
                    Err(MigrationError::InternalError {})
                },
                Ok(q) => Ok(q)
            }?;

            match conn
                .interact(move |conn| {
                    let mut statement = match conn.prepare(&query) {
                        Err(err) => {
                            error!(
                                error = err.to_string(),
                                migrate = file_name,
                                "error to execute prepare migrate query"
                            );
                            Err(MigrationError::PrepareStatementErr {})
                        }
                        Ok(s) => Ok(s),
                    }?;

                    match statement.execute([]) {
                        Err(err) => {
                            error!(error = err.to_string(), "error to execute migrate query");
                            Err(MigrationError::MigrateQueryErr {})
                        }
                        _ => Ok(()),
                    }?;

                    let mut statement =
                        match conn.prepare("INSERT INTO migrations (migrate) values (?)") {
                            Err(err) => {
                                error!(
                                    error = err.to_string(),
                                    "error to inset migrate in migrations table"
                                );
                                Err(MigrationError::InsertErr {})
                            }
                            Ok(s) => Ok(s),
                        }?;

                    match statement.execute([file_name]) {
                        Err(err) => {
                            error!(
                                error = err.to_string(),
                                "error to inset migrate in migrations table"
                            );
                            Err(MigrationError::InsertErr {})
                        }
                        _ => Ok(()),
                    }
                })
                .await
            {
                Err(_) => {
                    self.rollback(&conn).await?;
                    break;
                }
                _ => {}
            };
        }

        self.commit(&conn).await?;

        Ok(())
    }

    async fn down(
        &self,
        _path: Option<&str>,
        _migration: Option<&str>,
    ) -> Result<(), MigrationError> {
        Ok(())
    }
}

impl SqliteDriver {
    async fn get_conn(&self) -> Result<Object, MigrationError> {
        match self.pool.get().await {
            Err(err) => {
                error!(
                    error = err.to_string(),
                    "error to retrieve db connection from pool"
                );
                Err(MigrationError::DbConnectionErr {})
            }
            Ok(p) => Ok(p),
        }
    }

    async fn begin(&self, conn: &Object) -> Result<(), MigrationError> {
        match conn
            .interact(|conn| {
                let mut statement = match conn.prepare("BEGIN TRANSACTION migrations;") {
                    Err(e) => {
                        error!(error = e.to_string(), "error prepare begin transaction");
                        Err(MigrationError::InternalError {})
                    }
                    Ok(s) => Ok(s),
                }?;

                match statement.execute([]) {
                    Err(e) => {
                        error!(error = e.to_string(), "error query begin transaction");
                        Err(MigrationError::InternalError {})
                    }
                    _ => Ok(()),
                }
            })
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "unsuspected error");
                Err(MigrationError::InternalError {})
            }
            _ => Ok(()),
        }
    }

    async fn commit(&self, conn: &Object) -> Result<(), MigrationError> {
        conn.interact(|conn| {
            let mut statement = match conn.prepare("COMMIT TRANSACTION migrations;") {
                Err(e) => {
                    error!(error = e.to_string(), "error prepare commit transaction");
                    Err(MigrationError::InternalError {})
                }
                Ok(s) => Ok(s),
            }?;

            match statement.execute([]) {
                Err(e) => {
                    error!(error = e.to_string(), "error query commit transaction");
                    Err(MigrationError::InternalError {})
                }
                _ => Ok(()),
            }
        })
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "unsuspected error");
            MigrationError::InternalError {}
        })?
    }

    async fn rollback(&self, conn: &Object) -> Result<(), MigrationError> {
        conn.interact(|conn| {
            let mut statement = match conn.prepare("ROLLBACK TRANSACTION migrations;") {
                Err(e) => {
                    error!(error = e.to_string(), "error prepare rollback transaction");
                    Err(MigrationError::InternalError {})
                }
                Ok(s) => Ok(s),
            }?;

            match statement.execute([]) {
                Err(e) => {
                    error!(error = e.to_string(), "error query rollback transaction");
                    Err(MigrationError::InternalError {})
                }
                _ => Ok(()),
            }
        })
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "unsuspected error");
            MigrationError::InternalError {}
        })?
    }

    async fn migrate_executed_already(
        &self,
        conn: &Object,
        file_name: &String,
    ) -> Result<bool, MigrationError> {
        let file_name = file_name.clone();
        conn.interact(move |conn| {
            let mut statement =
                match conn.prepare("SELECT migrate FROM migrations WHERE migrate = ?;") {
                    Err(e) => {
                        error!(error = e.to_string(), "error prepare rollback transaction");
                        Err(MigrationError::InternalError {})
                    }
                    Ok(s) => Ok(s),
                }?;

            match statement.execute([&file_name]) {
                Err(e) => {
                    if e.sqlite_error_code().unwrap_or(ErrorCode::NotFound) != ErrorCode::NotFound {
                        error!(error = e.to_string(), "error query rollback transaction");
                        return Err(MigrationError::InternalError {});
                    }

                    Ok(false)
                }
                _ => Ok(true),
            }
        })
        .await
        .map_err(|e| {
            error!(error = e.to_string(), "unsuspected error");
            MigrationError::InternalError {}
        })?
    }
}
