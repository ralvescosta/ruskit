use crate::errors::MigrationError;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait MigratorDriver {
    async fn migration_table(&self) -> Result<(), MigrationError>;
    async fn up(&self, path: Option<&str>, migration: Option<&str>) -> Result<(), MigrationError>;
    async fn down(&self, path: Option<&str>, migration: Option<&str>)
        -> Result<(), MigrationError>;
}

#[derive(Default)]
pub enum MigrationMode {
    #[default]
    Up,
    Down,
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
        mode: &MigrationMode,
        path: Option<&str>,
        migration: Option<&str>,
    ) -> Result<(), MigrationError> {
        self.driver.migration_table().await?;

        match match mode {
            MigrationMode::Up => self.driver.up(path, migration).await,
            MigrationMode::Down => self.driver.down(path, migration).await,
        } {
            Err(e) => Err(e),
            _ => Ok(()),
        }?;

        Ok(())
    }

    pub async fn exec_up(
        &self,
        path: Option<&str>,
        migration: Option<&str>,
    ) -> Result<(), MigrationError> {
        self.exec(&MigrationMode::Up, path, migration).await
    }

    pub async fn exec_down(
        &self,
        path: Option<&str>,
        migration: Option<&str>,
    ) -> Result<(), MigrationError> {
        self.exec(&MigrationMode::Down, path, migration).await
    }
}
