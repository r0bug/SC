use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};

pub struct LocalStore {
    pool: Pool<Sqlite>,
}

impl LocalStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;
        Self::run_migrations(&pool).await?;
        Ok(Self { pool })
    }

    async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query(crate::migrations::SCHEMA).execute(pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}
