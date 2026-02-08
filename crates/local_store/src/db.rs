//! Database abstraction layer supporting SQLite and PostgreSQL
//!
//! Use feature flags to select backend:
//! - `sqlite` (default): Uses SQLite for local/offline storage
//! - `postgres`: Uses PostgreSQL for production/multi-user deployment

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub use sqlite_impl::*;

#[cfg(feature = "postgres")]
pub use postgres_impl::*;

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
mod sqlite_impl {
    use sqlx::{sqlite::SqlitePool, Pool, Sqlite};

    /// Database pool type alias
    pub type DbPool = Pool<Sqlite>;

    /// Concrete pool type for construction
    pub type ConcretePool = SqlitePool;

    /// Database backend identifier
    pub const DB_BACKEND: &str = "sqlite";

    /// Parameter placeholder style
    /// SQLite uses `?` for positional parameters
    pub fn param(index: usize) -> String {
        let _ = index; // SQLite doesn't use numbered params
        "?".to_string()
    }

    /// Connect to database
    pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
        SqlitePool::connect(database_url).await
    }

    /// Run database-specific initialization
    pub async fn initialize(pool: &DbPool) -> Result<(), sqlx::Error> {
        // Enable foreign key constraints for SQLite
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get the appropriate schema for this backend
    pub fn schema() -> &'static str {
        crate::migrations::SCHEMA_SQLITE
    }

    /// Convert boolean to database-compatible type
    pub fn bool_to_db(value: bool) -> i32 {
        value as i32
    }

    /// Convert database value to boolean
    pub fn db_to_bool(value: i32) -> bool {
        value != 0
    }
}

#[cfg(feature = "postgres")]
mod postgres_impl {
    use sqlx::{postgres::PgPool, Pool, Postgres};

    /// Database pool type alias
    pub type DbPool = Pool<Postgres>;

    /// Concrete pool type for construction
    pub type ConcretePool = PgPool;

    /// Database backend identifier
    pub const DB_BACKEND: &str = "postgres";

    /// Parameter placeholder style
    /// PostgreSQL uses `$1, $2, $3...` for positional parameters
    pub fn param(index: usize) -> String {
        format!("${}", index)
    }

    /// Connect to database
    pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
        PgPool::connect(database_url).await
    }

    /// Run database-specific initialization
    pub async fn initialize(_pool: &DbPool) -> Result<(), sqlx::Error> {
        // PostgreSQL doesn't need special initialization
        // Foreign keys are always enabled
        Ok(())
    }

    /// Get the appropriate schema for this backend
    pub fn schema() -> &'static str {
        crate::migrations::SCHEMA_POSTGRES
    }

    /// Convert boolean to database-compatible type (PostgreSQL uses native bool)
    pub fn bool_to_db(value: bool) -> bool {
        value
    }

    /// Convert database value to boolean
    pub fn db_to_bool(value: bool) -> bool {
        value
    }
}

/// Query builder helper for cross-database compatibility
pub struct QueryBuilder {
    query: String,
    param_count: usize,
}

impl QueryBuilder {
    pub fn new(base_query: &str) -> Self {
        Self {
            query: base_query.to_string(),
            param_count: 0,
        }
    }

    /// Get next parameter placeholder
    pub fn next_param(&mut self) -> String {
        self.param_count += 1;
        param(self.param_count)
    }

    /// Build query with proper parameter placeholders
    /// Takes a template with `{}` placeholders and fills them with database-appropriate params
    pub fn build_params(template: &str, param_count: usize) -> String {
        let mut result = template.to_string();
        for i in 1..=param_count {
            result = result.replacen("{}", &param(i), 1);
        }
        result
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_generation() {
        #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
        {
            assert_eq!(param(1), "?");
            assert_eq!(param(5), "?");
        }

        #[cfg(feature = "postgres")]
        {
            assert_eq!(param(1), "$1");
            assert_eq!(param(5), "$5");
        }
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::build_params("INSERT INTO test (a, b, c) VALUES ({}, {}, {})", 3);

        #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
        assert_eq!(query, "INSERT INTO test (a, b, c) VALUES (?, ?, ?)");

        #[cfg(feature = "postgres")]
        assert_eq!(query, "INSERT INTO test (a, b, c) VALUES ($1, $2, $3)");
    }
}
