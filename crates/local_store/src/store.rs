use anyhow::Result;
use crate::db::{DbPool, connect, initialize, schema, DB_BACKEND};
use tracing::info;

pub struct LocalStore {
    pool: DbPool,
}

impl LocalStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to {} database...", DB_BACKEND);
        let pool = connect(database_url).await?;
        initialize(&pool).await?;
        Self::run_migrations(&pool).await?;
        info!("Database initialized successfully ({})", DB_BACKEND);
        Ok(Self { pool })
    }

    async fn run_migrations(pool: &DbPool) -> Result<()> {
        // Run the appropriate schema for the backend
        let schema_sql = schema();

        // Split schema into individual statements and execute each
        // This handles the difference between SQLite (which supports multi-statement)
        // and PostgreSQL (which sometimes needs individual execution)
        for statement in schema_sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                sqlx::query(trimmed)
                    .execute(pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("Migration failed for statement: {}", trimmed);
                        e
                    })?;
            }
        }
        Ok(())
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    /// Get the database backend name
    pub fn backend(&self) -> &'static str {
        DB_BACKEND
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
        assert_eq!(DB_BACKEND, "sqlite");

        #[cfg(feature = "postgres")]
        assert_eq!(DB_BACKEND, "postgres");
    }
}
