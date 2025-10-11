use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetrics {
    pub id: Uuid,
    pub task_name: String,
    pub success_count: i64,
    pub failure_count: i64,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub average_duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct MetricsStore {
    pool: Arc<Pool<Sqlite>>,
}

impl MetricsStore {
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self { pool }
    }

    pub async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS worker_metrics (
                id TEXT PRIMARY KEY,
                task_name TEXT NOT NULL UNIQUE,
                success_count INTEGER DEFAULT 0,
                failure_count INTEGER DEFAULT 0,
                last_run_at TEXT,
                last_success_at TEXT,
                last_failure_at TEXT,
                last_error TEXT,
                average_duration_ms INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&*self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_worker_metrics_task_name
            ON worker_metrics(task_name);
            "#,
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_success(&self, task_name: &str, duration_ms: i64) -> Result<()> {
        let now = Utc::now();

        // Check if metrics exist for this task
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT id FROM worker_metrics WHERE task_name = ?")
                .bind(task_name)
                .fetch_optional(&*self.pool)
                .await?;

        if let Some((id,)) = existing {
            // Update existing metrics
            sqlx::query(
                r#"
                UPDATE worker_metrics
                SET success_count = success_count + 1,
                    last_run_at = ?,
                    last_success_at = ?,
                    average_duration_ms = (
                        CASE
                            WHEN average_duration_ms IS NULL THEN ?
                            ELSE (average_duration_ms * success_count + ?) / (success_count + 1)
                        END
                    ),
                    updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(duration_ms)
            .bind(duration_ms)
            .bind(now.to_rfc3339())
            .bind(id)
            .execute(&*self.pool)
            .await?;
        } else {
            // Create new metrics
            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO worker_metrics (
                    id, task_name, success_count, failure_count,
                    last_run_at, last_success_at, average_duration_ms,
                    created_at, updated_at
                ) VALUES (?, ?, 1, 0, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id.to_string())
            .bind(task_name)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(duration_ms)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn record_failure(&self, task_name: &str, error: &str) -> Result<()> {
        let now = Utc::now();

        // Check if metrics exist for this task
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT id FROM worker_metrics WHERE task_name = ?")
                .bind(task_name)
                .fetch_optional(&*self.pool)
                .await?;

        if let Some((id,)) = existing {
            // Update existing metrics
            sqlx::query(
                r#"
                UPDATE worker_metrics
                SET failure_count = failure_count + 1,
                    last_run_at = ?,
                    last_failure_at = ?,
                    last_error = ?,
                    updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(error)
            .bind(now.to_rfc3339())
            .bind(id)
            .execute(&*self.pool)
            .await?;
        } else {
            // Create new metrics
            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO worker_metrics (
                    id, task_name, success_count, failure_count,
                    last_run_at, last_failure_at, last_error,
                    created_at, updated_at
                ) VALUES (?, ?, 0, 1, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id.to_string())
            .bind(task_name)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(error)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_metrics(&self, task_name: &str) -> Result<Option<WorkerMetrics>> {
        let row: Option<(
            String,
            String,
            i64,
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            String,
            String,
        )> = sqlx::query_as(
            r#"
            SELECT id, task_name, success_count, failure_count,
                   last_run_at, last_success_at, last_failure_at,
                   last_error, average_duration_ms,
                   created_at, updated_at
            FROM worker_metrics
            WHERE task_name = ?
            "#,
        )
        .bind(task_name)
        .fetch_optional(&*self.pool)
        .await?;

        if let Some((
            id,
            task_name,
            success_count,
            failure_count,
            last_run_at,
            last_success_at,
            last_failure_at,
            last_error,
            average_duration_ms,
            created_at,
            updated_at,
        )) = row
        {
            Ok(Some(WorkerMetrics {
                id: Uuid::parse_str(&id)?,
                task_name,
                success_count,
                failure_count,
                last_run_at: last_run_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_success_at: last_success_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_failure_at: last_failure_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_error,
                average_duration_ms,
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_metrics(&self) -> Result<Vec<WorkerMetrics>> {
        let rows: Vec<(
            String,
            String,
            i64,
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            String,
            String,
        )> = sqlx::query_as(
            r#"
            SELECT id, task_name, success_count, failure_count,
                   last_run_at, last_success_at, last_failure_at,
                   last_error, average_duration_ms,
                   created_at, updated_at
            FROM worker_metrics
            ORDER BY task_name
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut metrics = Vec::new();
        for (
            id,
            task_name,
            success_count,
            failure_count,
            last_run_at,
            last_success_at,
            last_failure_at,
            last_error,
            average_duration_ms,
            created_at,
            updated_at,
        ) in rows
        {
            metrics.push(WorkerMetrics {
                id: Uuid::parse_str(&id)?,
                task_name,
                success_count,
                failure_count,
                last_run_at: last_run_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_success_at: last_success_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_failure_at: last_failure_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_error,
                average_duration_ms,
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            });
        }

        Ok(metrics)
    }
}
