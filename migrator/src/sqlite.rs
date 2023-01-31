use crate::service::MigratorDriver;
use async_trait::async_trait;
use errors::migrator::MigrationError;
use std::{fs, sync::Arc};
use tracing::{debug, error, warn};
use deadpool_sqlite::{Config, Pool, Runtime, Object, Manager};
use env::SqliteConfig;

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
        // let conn = self.get_conn().await?;

        // self.begin(&conn)?;

        // let query = "SELECT migrate FROM migrations limit 1";
        // let Err(err) = conn.prepare(query) else {
        //     debug!("migration table already created");
        //     self.commit(&conn)?;
        //     return Ok(());
        // };

        // if err.sqlite_error_code().unwrap_or(ErrorCode::NotFound) != ErrorCode::Unknown {
        //     error!(error = err.to_string(), "unexpected error");
        //     self.rollback(&conn)?;
        //     return Err(MigrationError::InternalError {});
        // }

        // let query = "
        //     CREATE TABLE migrations (
        //         id INTEGER PRIMARY KEY AUTOINCREMENT,
        //         migrate TEXT,
        //         executed_at TIMESTAMPZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
        //         rollback_at TIMESTAMPZ
        //     )";

        // let Ok(mut statement) = conn.prepare(query) else {
        //     error!("error to prepare create migrations table query");
        //     self.rollback(&conn)?;
        //     return Err(MigrationError::CreateMigrationsTableErr {});
        // };

        // let Ok(_) = statement.execute([]) else {
        //     error!("error to execute create migrations table query");
        //     self.rollback(&conn)?;
        //     return Err(MigrationError::CreateMigrationsTableErr {});
        // };

        // self.commit(&conn)?;

        Ok(())
    }

    async fn up(
        &self,
        path: Option<&String>,
        _migration: Option<&String>,
    ) -> Result<(), MigrationError> {
        // let mut migrations_path = "./bins/migrations/sql/";

        // if path.is_some() {
        //     migrations_path = path.unwrap().as_str();
        // }

        // let conn = self.get_conn().await?;

        // self.begin(&conn)?;

        // for res_dir_entry in fs::read_dir(migrations_path).unwrap() {
        //     let dir_entry = res_dir_entry.unwrap();

        //     let file_name = dir_entry.file_name().into_string().unwrap_or_default();

        //     if !file_name.contains("up") || !file_name.contains(".sql") {
        //         warn!(file = file_name, "skipping migrate");
        //         continue;
        //     }

        //     if self.migrate_executed_already(&conn, &file_name)? {
        //         continue;
        //     }

        //     let query: String = fs::read_to_string(dir_entry.path()).map_err(|e| {
        //         error!(error = e.to_string(), "error to read migration file");
        //         MigrationError::InternalError {}
        //     })?;

        //     let mut statement = match conn.prepare(&query) {
        //         Err(err) => {
        //             error!(
        //                 error = err.to_string(),
        //                 migrate = file_name,
        //                 "error to execute prepare migrate query"
        //             );
        //             self.rollback(&conn)?;
        //             Err(MigrationError::PrepareStatementErr {})
        //         }
        //         Ok(s) => Ok(s),
        //     }?;

        //     match statement.execute([]) {
        //         Err(err) => {
        //             error!(error = err.to_string(), "error to execute migrate query");
        //             self.rollback(&conn)?;
        //             Err(MigrationError::MigrateQueryErr {})
        //         }
        //         _ => Ok(()),
        //     }?;

        //     let mut statement = match conn.prepare("INSERT INTO migrations (migrate) values (?)") {
        //         Err(err) => {
        //             error!(
        //                 error = err.to_string(),
        //                 "error to inset migrate in migrations table"
        //             );
        //             self.rollback(&conn)?;
        //             Err(MigrationError::InsertErr {})
        //         }
        //         Ok(s) => Ok(s),
        //     }?;

        //     match statement.execute([file_name]) {
        //         Err(err) => {
        //             error!(
        //                 error = err.to_string(),
        //                 "error to inset migrate in migrations table"
        //             );
        //             self.rollback(&conn)?;
        //             Err(MigrationError::InsertErr {})
        //         }
        //         _ => Ok(()),
        //     }?;
        // }

        // self.commit(&conn)?;

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

    async fn begin(
        &self,
        conn: &Object,
    ) -> Result<(), MigrationError> {
        conn.interact(|conn| {
            let mut statement = match conn.prepare("BEGIN TRANSACTION migrations;"){
                Err(e) => {
                    error!(error = e.to_string(), "error prepare begin transaction");
                   Err(MigrationError::InternalError {})
                },
                Ok(s) => Ok(s),
            }?;
            
            match statement.execute([]) {
                Err(e) => {
                    error!(error = e.to_string(), "error query begin transaction");
                    Err(MigrationError::InternalError {})
                },
                _ => Ok(())
            }
        }).await.map_err(|_| {
            MigrationError::InternalError {}
        })?
    }

    fn commit(
        &self,
        conn: &Object,
    ) -> Result<(), MigrationError> {
        // conn.prepare("COMMIT TRANSACTION migrations;")
        //     .map_err(|e| {
        //         error!(error = e.to_string(), "error prepare commit transaction");
        //         MigrationError::InternalError {}
        //     })?
        //     .execute([])
        //     .map_err(|e| {
        //         error!(error = e.to_string(), "error query commit transaction");
        //         MigrationError::InternalError {}
        //     })?;

        Ok(())
    }

    fn rollback(
        &self,
        conn: &Object,
    ) -> Result<(), MigrationError> {
        // conn.prepare("ROLLBACK TRANSACTION migrations;")
        //     .map_err(|e| {
        //         error!(error = e.to_string(), "error prepare rollback transaction");
        //         MigrationError::InternalError {}
        //     })?
        //     .execute([])
        //     .map_err(|e| {
        //         error!(error = e.to_string(), "error query rollback transaction");
        //         MigrationError::InternalError {}
        //     })?;

        Ok(())
    }

    fn migrate_executed_already(
        &self,
        conn: &Object,
        file_name: &String,
    ) -> Result<bool, MigrationError> {
        // let res: Result<String, _> = conn.query_row(
        //     "SELECT migrate FROM migrations WHERE migrate = ?",
        //     [file_name],
        //     |v| v.get(0),
        // );

        // match res {
        //     Err(e) => {
        //         if e.sqlite_error_code().unwrap_or(ErrorCode::NotFound) != ErrorCode::NotFound {
        //             error!(error = e.to_string(), "unsuspected error");
        //             return Err(MigrationError::InternalError {});
        //         }
        //         Ok(false)
        //     }
        //     _ => Ok(true),
        // }
        Ok(true)
    }
}
