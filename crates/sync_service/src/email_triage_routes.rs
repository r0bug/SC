use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use local_store::repositories::{
    CommunicationConceptRepository, ConceptRepository, EmailHistoryRepository,
    EmailTriageSessionRepository,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::email_triage::EmailTriageService;
use crate::state::AppState;

// ─── Request / Response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct StartTriageRequest {
    pub server: String,
    pub folder: Option<String>,
    pub block_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewDomainsRequest {
    pub approved: Vec<String>, // concept IDs to approve
    pub denied: Vec<String>,   // concept IDs to deny
}

#[derive(Debug, Deserialize)]
pub struct ContinueTriageRequest {
    pub auto_apply: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FilterRequest {
    pub domain_ids: Vec<String>,
    pub concept_ids: Option<Vec<String>>,
    pub mode: String, // "union" or "intersection"
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ─── Route builder ───────────────────────────────────────────────────

pub fn email_triage_routes() -> Router<AppState> {
    Router::new()
        // Triage wizard endpoints
        .route("/api/email/triage/start", post(start_triage))
        .route("/api/email/triage/:id", get(get_triage_session))
        .route("/api/email/triage/:id/proposals", get(get_triage_proposals))
        .route("/api/email/triage/:id/review", post(review_triage_domains))
        .route("/api/email/triage/:id/continue", post(continue_triage))
        // Domain view endpoints
        .route("/api/email/domains", get(list_email_domains))
        .route("/api/email/domains/filter", post(filter_emails_by_domains))
        .route(
            "/api/email/domains/:concept_id/emails",
            get(get_domain_emails),
        )
        .route(
            "/api/email/domains/:concept_id/stats",
            get(get_domain_stats),
        )
}

// ─── Triage Endpoints ────────────────────────────────────────────────

/// POST /api/email/triage/start
pub async fn start_triage(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<StartTriageRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let folder = req.folder.unwrap_or_else(|| "INBOX".to_string());
    let block_size = req.block_size.unwrap_or(25);

    let service = EmailTriageService::new(&state.pool, state.ai_client.read().await.clone());

    let session = service
        .start_session(user.id, &req.server, &folder, block_size)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Analyze first block
    let result = service
        .analyze_block(session.id, 0, block_size, &[], user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Update session status to "reviewing"
    let session_repo = EmailTriageSessionRepository::new(&state.pool);
    session_repo
        .update_status(session.id, "reviewing", None)
        .await
        .ok();

    // Store discovered domains in session
    let discovered: Vec<String> = result
        .proposals
        .iter()
        .map(|p| p.concept_id.clone())
        .collect();
    let discovered_json = serde_json::to_string(&discovered).unwrap_or_else(|_| "[]".to_string());
    session_repo
        .update_domains(session.id, &discovered_json, "[]", "[]")
        .await
        .ok();

    Ok(Json(serde_json::json!({
        "session_id": session.id.to_string(),
        "status": "reviewing",
        "total_fetched": session.total_fetched,
        "block_size": block_size,
        "total_blocks": (session.total_fetched as f64 / block_size as f64).ceil() as i64,
        "current_block": 0,
        "proposals": result.proposals,
    })))
}

/// GET /api/email/triage/:id
pub async fn get_triage_session(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session_repo = EmailTriageSessionRepository::new(&state.pool);

    let session = session_repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": session.id.to_string(),
        "status": session.status,
        "server": session.server,
        "folder": session.folder,
        "total_fetched": session.total_fetched,
        "total_analyzed": session.total_analyzed,
        "total_categorized": session.total_categorized,
        "current_block": session.current_block,
        "block_size": session.block_size,
        "total_blocks": (session.total_fetched as f64 / session.block_size as f64).ceil() as i64,
        "discovered_domains": serde_json::from_str::<serde_json::Value>(&session.discovered_domains).unwrap_or_default(),
        "approved_domains": serde_json::from_str::<serde_json::Value>(&session.approved_domains).unwrap_or_default(),
        "denied_domains": serde_json::from_str::<serde_json::Value>(&session.denied_domains).unwrap_or_default(),
        "error_message": session.error_message,
        "created_at": session.created_at.to_rfc3339(),
        "updated_at": session.updated_at.to_rfc3339(),
        "completed_at": session.completed_at.map(|dt| dt.to_rfc3339()),
    })))
}

/// GET /api/email/triage/:id/proposals
pub async fn get_triage_proposals(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session_repo = EmailTriageSessionRepository::new(&state.pool);
    let concept_repo = ConceptRepository::new(&state.pool);
    let cc_repo = CommunicationConceptRepository::new(&state.pool);

    let session = session_repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Parse discovered domain IDs
    let domain_ids: Vec<String> =
        serde_json::from_str(&session.discovered_domains).unwrap_or_default();

    let mut proposals = Vec::new();
    for domain_id_str in &domain_ids {
        let concept_id = Uuid::parse_str(domain_id_str).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid UUID: {}", e),
            )
        })?;

        if let Ok(concept) = concept_repo.get_by_id(concept_id).await {
            // Get linked emails count
            let linked = cc_repo
                .list_by_concept(concept_id)
                .await
                .unwrap_or_default();

            let email_count = linked
                .iter()
                .filter(|c| c.communication_type == "email")
                .count();

            let keywords: Vec<String> =
                concept.keywords.iter().map(|k| k.keyword.clone()).collect();

            proposals.push(serde_json::json!({
                "concept_id": concept.id.to_string(),
                "name": concept.name,
                "description": concept.description,
                "keywords": keywords,
                "email_count": email_count,
                "confidence": concept.metadata.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5),
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "session_id": id.to_string(),
        "proposals": proposals,
    })))
}

/// POST /api/email/triage/:id/review
pub async fn review_triage_domains(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewDomainsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session_repo = EmailTriageSessionRepository::new(&state.pool);
    let cc_repo = CommunicationConceptRepository::new(&state.pool);

    let session = session_repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Confirm approved domains - update all their communication_concepts to "confirmed"
    for concept_id_str in &req.approved {
        if let Ok(concept_id) = Uuid::parse_str(concept_id_str) {
            let links = cc_repo
                .list_by_concept(concept_id)
                .await
                .unwrap_or_default();
            for link in links {
                if link.status == core_domain::SuggestionStatus::Suggested {
                    cc_repo
                        .update_status(link.id, &core_domain::SuggestionStatus::Confirmed, user.id)
                        .await
                        .ok();
                }
            }
        }
    }

    // Deny denied domains - update all their communication_concepts to "denied"
    for concept_id_str in &req.denied {
        if let Ok(concept_id) = Uuid::parse_str(concept_id_str) {
            let links = cc_repo
                .list_by_concept(concept_id)
                .await
                .unwrap_or_default();
            for link in links {
                if link.status == core_domain::SuggestionStatus::Suggested {
                    cc_repo
                        .update_status(link.id, &core_domain::SuggestionStatus::Denied, user.id)
                        .await
                        .ok();
                }
            }
        }
    }

    // Update session domains
    let approved_json = serde_json::to_string(&req.approved).unwrap_or_else(|_| "[]".to_string());
    let denied_json = serde_json::to_string(&req.denied).unwrap_or_else(|_| "[]".to_string());
    session_repo
        .update_domains(
            id,
            &session.discovered_domains,
            &approved_json,
            &denied_json,
        )
        .await
        .ok();

    Ok(Json(serde_json::json!({
        "session_id": id.to_string(),
        "approved_count": req.approved.len(),
        "denied_count": req.denied.len(),
    })))
}

/// POST /api/email/triage/:id/continue
pub async fn continue_triage(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ContinueTriageRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session_repo = EmailTriageSessionRepository::new(&state.pool);

    let session = session_repo
        .get_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let next_block = session.current_block + 1;
    let total_blocks = (session.total_fetched as f64 / session.block_size as f64).ceil() as i64;

    let auto_apply = req.auto_apply.unwrap_or(false);

    // If auto_apply is set and we have approved domains, apply them to remaining emails
    if auto_apply {
        let approved_ids: Vec<String> =
            serde_json::from_str(&session.approved_domains).unwrap_or_default();
        let approved_uuids: Vec<Uuid> = approved_ids
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        if !approved_uuids.is_empty() {
            let service =
                EmailTriageService::new(&state.pool, state.ai_client.read().await.clone());
            let categorized = service
                .apply_approved_domains(id, &approved_uuids)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

            session_repo.update_status(id, "completed", None).await.ok();

            return Ok(Json(serde_json::json!({
                "session_id": id.to_string(),
                "status": "completed",
                "total_categorized": categorized,
                "auto_applied": true,
            })));
        }
    }

    // Otherwise analyze the next block
    if next_block >= total_blocks {
        session_repo.update_status(id, "completed", None).await.ok();

        return Ok(Json(serde_json::json!({
            "session_id": id.to_string(),
            "status": "completed",
            "message": "All blocks processed",
        })));
    }

    let approved_ids: Vec<String> =
        serde_json::from_str(&session.approved_domains).unwrap_or_default();

    let service = EmailTriageService::new(&state.pool, state.ai_client.read().await.clone());
    let result = service
        .analyze_block(id, next_block, session.block_size, &approved_ids, user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Update discovered domains to include new proposals
    let mut all_discovered: Vec<String> =
        serde_json::from_str(&session.discovered_domains).unwrap_or_default();
    for proposal in &result.proposals {
        if !all_discovered.contains(&proposal.concept_id) {
            all_discovered.push(proposal.concept_id.clone());
        }
    }
    let discovered_json =
        serde_json::to_string(&all_discovered).unwrap_or_else(|_| "[]".to_string());
    session_repo
        .update_domains(
            id,
            &discovered_json,
            &session.approved_domains,
            &session.denied_domains,
        )
        .await
        .ok();

    session_repo.update_status(id, "reviewing", None).await.ok();

    Ok(Json(serde_json::json!({
        "session_id": id.to_string(),
        "status": "reviewing",
        "current_block": next_block,
        "total_blocks": total_blocks,
        "proposals": result.proposals,
    })))
}

// ─── Domain View Endpoints ───────────────────────────────────────────

/// GET /api/email/domains
pub async fn list_email_domains(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let concept_repo = ConceptRepository::new(&state.pool);

    let domains = concept_repo
        .list_communication_domains()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut domain_list = Vec::new();
    for domain in &domains {
        // Get email count for each domain
        let service = EmailTriageService::new(&state.pool, state.ai_client.read().await.clone());
        let stats = service
            .get_domain_stats(domain.id)
            .await
            .unwrap_or_else(|_| crate::email_triage::DomainStats {
                concept_id: domain.id.to_string(),
                name: domain.name.clone(),
                email_count: 0,
                unique_senders: 0,
                earliest_date: None,
                latest_date: None,
                top_senders: Vec::new(),
            });

        let keywords: Vec<String> = domain.keywords.iter().map(|k| k.keyword.clone()).collect();

        domain_list.push(serde_json::json!({
            "concept_id": domain.id.to_string(),
            "name": domain.name,
            "description": domain.description,
            "keywords": keywords,
            "email_count": stats.email_count,
            "unique_senders": stats.unique_senders,
            "earliest_date": stats.earliest_date,
            "latest_date": stats.latest_date,
        }));
    }

    Ok(Json(serde_json::json!({
        "domains": domain_list,
        "total": domain_list.len(),
    })))
}

/// GET /api/email/domains/:concept_id/emails
pub async fn get_domain_emails(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(concept_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email_repo = EmailHistoryRepository::new(&state.pool);

    let emails = email_repo
        .get_by_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let messages: Vec<serde_json::Value> = emails
        .iter()
        .map(|msg| {
            serde_json::json!({
                "id": msg.id.to_string(),
                "from_address": msg.from_address,
                "from_name": msg.from_name,
                "to_address": msg.to_address,
                "subject": msg.subject,
                "body_text": msg.body_text,
                "body_html": msg.body_html,
                "message_date": msg.message_date,
                "message_type": msg.message_type,
                "read_status": msg.read_status,
                "has_attachments": msg.has_attachments,
                "folder": msg.folder,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "concept_id": concept_id.to_string(),
        "emails": messages,
        "total": messages.len(),
    })))
}

/// GET /api/email/domains/:concept_id/stats
pub async fn get_domain_stats(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(concept_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let service = EmailTriageService::new(&state.pool, state.ai_client.read().await.clone());

    let stats = service
        .get_domain_stats(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(stats))
}

/// POST /api/email/domains/filter
pub async fn filter_emails_by_domains(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<FilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email_repo = EmailHistoryRepository::new(&state.pool);

    // Combine domain_ids and optional concept_ids
    let mut all_concept_ids: Vec<Uuid> = Vec::new();
    for id_str in &req.domain_ids {
        let uid = Uuid::parse_str(id_str).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid domain UUID: {}", e),
            )
        })?;
        all_concept_ids.push(uid);
    }
    if let Some(ref extra_ids) = req.concept_ids {
        for id_str in extra_ids {
            let uid = Uuid::parse_str(id_str).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid concept UUID: {}", e),
                )
            })?;
            all_concept_ids.push(uid);
        }
    }

    let mode = if req.mode == "intersection" {
        "intersection"
    } else {
        "union"
    };

    let limit = req.limit.unwrap_or(50);
    let offset = req.offset.unwrap_or(0);

    let results = email_repo
        .filter_by_concepts(&all_concept_ids, mode, limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get overlap counts for Venn diagram
    let overlaps = email_repo
        .get_domain_overlap_counts(&all_concept_ids)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "emails": results,
        "total": results.len(),
        "mode": mode,
        "overlaps": overlaps,
    })))
}
