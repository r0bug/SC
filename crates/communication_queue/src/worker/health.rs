use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use anyhow::Result;

use super::{supervisor::WorkerSupervisor, metrics::MetricsStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tasks: TasksStatus,
    pub metrics: Vec<TaskMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksStatus {
    pub total: usize,
    pub running: usize,
    pub failed: usize,
    pub stopped: usize,
    pub details: Vec<TaskDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: String,
    pub name: String,
    pub status: String,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub task_name: String,
    pub success_count: i64,
    pub failure_count: i64,
    pub last_run: Option<String>,
    pub average_duration_ms: Option<i64>,
}

pub struct HealthServer {
    supervisor: Arc<WorkerSupervisor>,
    metrics_store: Arc<MetricsStore>,
    port: u16,
}

impl HealthServer {
    pub fn new(
        supervisor: Arc<WorkerSupervisor>,
        metrics_store: Arc<MetricsStore>,
    ) -> Self {
        Self {
            supervisor,
            metrics_store,
            port: 9090, // Default health check port
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub async fn start(self) -> Result<()> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/health/worker", get(worker_health_handler))
            .with_state(AppState {
                supervisor: self.supervisor,
                metrics_store: self.metrics_store,
            });

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("Health server listening on http://0.0.0.0:{}", self.port);

        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    supervisor: Arc<WorkerSupervisor>,
    metrics_store: Arc<MetricsStore>,
}

async fn health_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_report = state.supervisor.health_check().await;

    let status = if health_report.healthy {
        "healthy"
    } else {
        "unhealthy"
    };

    Ok(Json(serde_json::json!({
        "status": status,
        "timestamp": chrono::Utc::now(),
        "tasks": {
            "total": health_report.total_tasks,
            "running": health_report.running_tasks,
            "failed": health_report.failed_tasks,
        }
    })))
}

async fn worker_health_handler(
    State(state): State<AppState>,
) -> Result<Json<HealthStatus>, StatusCode> {
    let health_report = state.supervisor.health_check().await;

    // Get task details
    let task_details: Vec<TaskDetail> = health_report
        .tasks
        .iter()
        .map(|task| TaskDetail {
            id: task.id.to_string(),
            name: task.name.clone(),
            status: format!("{:?}", task.status),
            restart_count: task.restart_count,
            last_error: task.last_error.clone(),
        })
        .collect();

    // Get metrics for all tasks
    let all_metrics = state.metrics_store.get_all_metrics().await
        .unwrap_or_else(|_| Vec::new());

    let task_metrics: Vec<TaskMetrics> = all_metrics
        .into_iter()
        .map(|m| TaskMetrics {
            task_name: m.task_name,
            success_count: m.success_count,
            failure_count: m.failure_count,
            last_run: m.last_run_at.map(|dt| dt.to_rfc3339()),
            average_duration_ms: m.average_duration_ms,
        })
        .collect();

    let status = if health_report.healthy {
        "healthy"
    } else {
        "unhealthy"
    };

    let health_status = HealthStatus {
        status: status.to_string(),
        timestamp: chrono::Utc::now(),
        tasks: TasksStatus {
            total: health_report.total_tasks,
            running: health_report.running_tasks,
            failed: health_report.failed_tasks,
            stopped: health_report.total_tasks - health_report.running_tasks - health_report.failed_tasks,
            details: task_details,
        },
        metrics: task_metrics,
    };

    Ok(Json(health_status))
}