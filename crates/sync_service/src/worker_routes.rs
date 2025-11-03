use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use core_domain::CommunicationStatus as DomainCommunicationStatus;
use local_store::repositories::CommunicationRepository;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CommunicationStatusUpdate {
    pub status: CommunicationStatus,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommunicationStatus {
    Queued,
    Processing,
    Sent,
    Delivered,
    Failed,
    Bounced,
}

impl CommunicationStatus {
    fn to_domain(&self) -> DomainCommunicationStatus {
        match self {
            CommunicationStatus::Queued => DomainCommunicationStatus::Pending,
            CommunicationStatus::Processing => DomainCommunicationStatus::Retrying,
            CommunicationStatus::Sent => DomainCommunicationStatus::Sent,
            CommunicationStatus::Delivered => DomainCommunicationStatus::Sent,
            CommunicationStatus::Failed => DomainCommunicationStatus::Failed {
                reason: "Worker reported failure".to_string(),
            },
            CommunicationStatus::Bounced => DomainCommunicationStatus::Failed {
                reason: "Message bounced".to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub message: String,
    pub communication_id: Uuid,
}

/// POST /api/communication/:id/status
/// Worker callback endpoint to update communication status
pub async fn update_communication_status(
    State(app_state): State<AppState>,
    Path(communication_id): Path<Uuid>,
    Json(update): Json<CommunicationStatusUpdate>,
) -> Result<Json<StatusResponse>, (StatusCode, &'static str)> {
    tracing::info!(
        "Communication {} status updated to {:?}",
        communication_id,
        update.status
    );

    if let Some(message) = &update.message {
        tracing::info!("Status message: {}", message);
    }

    // Persist the status update to the database
    let repo = CommunicationRepository::new(app_state.store.pool());
    let domain_status = update.status.to_domain();
    let attempted_at = Some(Utc::now());

    repo.update_status(communication_id, domain_status, attempted_at)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update communication status: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update status")
        })?;

    Ok(Json(StatusResponse {
        message: format!("Status updated to {:?}", update.status),
        communication_id,
    }))
}

/// POST /api/communication/batch-status
/// Worker callback endpoint to update multiple communication statuses at once
pub async fn batch_update_communication_status(
    State(app_state): State<AppState>,
    Json(updates): Json<Vec<BatchStatusUpdate>>,
) -> Result<Json<BatchStatusResponse>, (StatusCode, &'static str)> {
    let count = updates.len();
    let repo = CommunicationRepository::new(app_state.store.pool());
    let attempted_at = Some(Utc::now());

    let mut processed = 0;
    for update in updates {
        tracing::info!(
            "Batch update: Communication {} status -> {:?}",
            update.communication_id,
            update.status
        );

        // Persist each status update to the database
        let domain_status = update.status.to_domain();
        match repo.update_status(update.communication_id, domain_status, attempted_at).await {
            Ok(_) => processed += 1,
            Err(e) => {
                tracing::error!(
                    "Failed to update communication {} status: {:?}",
                    update.communication_id,
                    e
                );
            }
        }
    }

    Ok(Json(BatchStatusResponse {
        message: format!("Updated {} of {} communication statuses", processed, count),
        processed,
    }))
}

#[derive(Debug, Deserialize)]
pub struct BatchStatusUpdate {
    pub communication_id: Uuid,
    pub status: CommunicationStatus,
}

#[derive(Debug, Serialize)]
pub struct BatchStatusResponse {
    pub message: String,
    pub processed: usize,
}

/// POST /api/worker/health
/// Health check endpoint for worker services
pub async fn worker_health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now(),
    }))
}

/// POST /api/worker/register
/// Register a worker with the sync service
pub async fn register_worker(
    State(_app_state): State<AppState>,
    Json(registration): Json<WorkerRegistration>,
) -> Result<Json<WorkerRegistrationResponse>, (StatusCode, &'static str)> {
    tracing::info!("Worker registered: {}", registration.worker_id);

    Ok(Json(WorkerRegistrationResponse {
        worker_id: registration.worker_id,
        registered: true,
        assigned_queues: vec!["email".to_string(), "sms".to_string()],
    }))
}

#[derive(Debug, Deserialize)]
pub struct WorkerRegistration {
    pub worker_id: String,
}

#[derive(Debug, Serialize)]
pub struct WorkerRegistrationResponse {
    pub worker_id: String,
    pub registered: bool,
    pub assigned_queues: Vec<String>,
}
