use async_trait::async_trait;
use errors::migrator::MigrationError;
use std::sync::Arc;

#[async_trait]
pub trait MigratorDriver {
    async fn migration_table(&self) -> Result<(), MigrationError>;
    async fn up(
        &self,
        path: Option<&String>,
        migration: Option<&String>,
    ) -> Result<(), MigrationError>;
    async fn down(
        &self,
        path: Option<&String>,
        migration: Option<&String>,
    ) -> Result<(), MigrationError>;
}

pub struct Migrator {
    driver: Arc<dyn MigratorDriver>,
}

impl Migrator {
    pub fn new(driver: Arc<dyn MigratorDriver>) -> Migrator {
        Migrator { driver }
    }
}

impl Migrator {
    pub async fn exec(
        &self,
        mode: &String,
        path: Option<&String>,
        migration: Option<&String>,
    ) -> Result<(), MigrationError> {
        self.driver.migration_table().await?;

        match match mode.as_str() {
            "up" => self.driver.up(path, migration).await,
            "down" => self.driver.down(path, migration).await,
            _ => Err(MigrationError::InvalidArgumentErr(mode.to_owned())),
        } {
            Err(e) => Err(e),
            _ => Ok(()),
        }?;

        Ok(())
    }

    pub async fn exec_up(
        &self,
        path: Option<&String>,
        migration: Option<&String>,
    ) -> Result<(), MigrationError> {
        self.exec(&"up".to_owned(), path, migration).await
    }

    pub async fn exec_down(
        &self,
        path: Option<&String>,
        migration: Option<&String>,
    ) -> Result<(), MigrationError> {
        self.exec(&"down".to_owned(), path, migration).await
    }
}
