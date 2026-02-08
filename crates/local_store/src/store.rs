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
        // Run the appropriate base schema for the backend
        Self::execute_sql(pool, schema(), false).await?;

        // Run upgrade migrations (ALTER TABLE statements for existing databases).
        // On fresh installs these will fail with "duplicate column" since the
        // CREATE TABLE already includes the columns — that's expected and ignored.
        Self::execute_sql(pool, crate::migrations::UPGRADE_MIGRATIONS, true).await?;

        Ok(())
    }

    /// Execute a block of SQL statements separated by ';'.
    /// If `tolerate_alter_errors` is true, ALTER TABLE "duplicate column" errors
    /// are silently ignored (for upgrade migrations on fresh installs).
    async fn execute_sql(pool: &DbPool, sql: &str, tolerate_alter_errors: bool) -> Result<()> {
        // Strip SQL comment lines first (before splitting on ';')
        // to avoid ';' inside comments creating bogus statements
        let cleaned: String = sql
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        for statement in cleaned.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                match sqlx::query(trimmed).execute(pool).await {
                    Ok(_) => {}
                    Err(e) => {
                        let err_msg = e.to_string();
                        // ALTER TABLE ADD COLUMN fails with "duplicate column" on
                        // fresh installs where CREATE TABLE already included the column.
                        // This is expected and safe to ignore.
                        let is_alter_table = trimmed.to_uppercase().starts_with("ALTER TABLE");
                        if tolerate_alter_errors && is_alter_table && err_msg.contains("duplicate column") {
                            tracing::debug!("Skipping already-applied migration: {}", trimmed);
                        } else {
                            tracing::error!("Migration failed for statement: {}", trimmed);
                            return Err(e.into());
                        }
                    }
                }
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
