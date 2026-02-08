use crate::{auth::AuthUser, state::AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use core_domain::{
    CommunicationConcept, Concept, ConceptKeyword, DetectionMethod, SuggestionStatus,
};
use local_store::repositories::{CommunicationConceptRepository, ConceptRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub name: String,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AssignDomainsRequest {
    pub domain_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkAssignRequest {
    pub email_ids: Vec<Uuid>,
    pub domain_names: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DomainSearchResult {
    pub concept_id: Uuid,
    pub name: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DomainAssignment {
    pub concept_id: Uuid,
    pub name: String,
    pub status: String,
    pub detection_method: String,
}

#[derive(Debug, Serialize)]
pub struct AssignResponse {
    pub assigned: Vec<DomainAssignment>,
    pub created: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkAssignResponse {
    pub assigned_count: usize,
    pub created_domains: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatedDomain {
    pub concept_id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SenderAssignResponse {
    pub assigned_count: usize,
    pub message_count: usize,
    pub created_domains: Vec<String>,
}

// --- Router ---

pub fn manual_domain_routes() -> Router<AppState> {
    Router::new()
        .route("/api/email/domains/search", get(search_domains))
        .route("/api/email/domains/create", post(create_domain))
        .route(
            "/api/email/:email_id/domains",
            get(get_email_domains).post(assign_domains),
        )
        .route(
            "/api/email/:email_id/domains/:concept_id",
            axum::routing::delete(unlink_domain),
        )
        .route(
            "/api/email/bulk-assign-domains",
            post(bulk_assign_domains),
        )
        // Sender-level domain endpoints
        .route(
            "/api/email/sender/:address/domains",
            get(get_sender_domains).post(assign_domains_to_sender),
        )
        .route(
            "/api/sms/sender/:phone/domains",
            get(get_sms_sender_domains).post(assign_domains_to_sms_sender),
        )
        .route(
            "/api/sms/:sms_id/domains",
            get(get_sms_domains),
        )
}

// --- Handlers ---

/// GET /api/email/domains/search?q=...
async fn search_domains(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Query(params): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let concept_repo = ConceptRepository::new(&app_state.pool);

    let domains = match &params.q {
        Some(q) if !q.trim().is_empty() => concept_repo
            .search_communication_domains(q.trim())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        _ => concept_repo
            .list_communication_domains()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    let results: Vec<DomainSearchResult> = domains
        .into_iter()
        .map(|c| DomainSearchResult {
            concept_id: c.id,
            name: c.name,
            keywords: c.keywords.into_iter().map(|k| k.keyword).collect(),
        })
        .collect();

    Ok(Json(serde_json::json!({ "domains": results })))
}

/// POST /api/email/domains/create
async fn create_domain(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateDomainRequest>,
) -> Result<Json<CreatedDomain>, (StatusCode, String)> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Domain name is required".to_string()));
    }

    let concept_repo = ConceptRepository::new(&app_state.pool);

    // Check for existing domain with same name
    if let Some(existing) = concept_repo
        .find_communication_domain_by_name(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        return Ok(Json(CreatedDomain {
            concept_id: existing.id,
            name: existing.name,
        }));
    }

    let now = Utc::now();
    let concept_id = Uuid::new_v4();

    let keywords: Vec<ConceptKeyword> = req
        .keywords
        .unwrap_or_default()
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .map(|k| ConceptKeyword {
            id: Uuid::new_v4(),
            concept_id,
            keyword: k.trim().to_string(),
            created_at: now,
        })
        .collect();

    let concept = Concept {
        id: concept_id,
        name: name.clone(),
        description: req.description,
        parent_id: None,
        keywords,
        metadata: serde_json::json!({ "concept_type": "communication_domain" }),
        created_at: now,
        updated_at: now,
        created_by: user.id,
    };

    concept_repo
        .create(&concept)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CreatedDomain {
        concept_id,
        name,
    }))
}

/// GET /api/email/:email_id/domains
async fn get_email_domains(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(email_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let concept_repo = ConceptRepository::new(&app_state.pool);

    let links = cc_repo
        .list_by_communication("email", email_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut domains = Vec::new();
    for link in &links {
        if let Ok(concept) = concept_repo.get_by_id(link.concept_id).await {
            domains.push(DomainAssignment {
                concept_id: concept.id,
                name: concept.name,
                status: link.status.to_string(),
                detection_method: link.detection_method.to_string(),
            });
        }
    }

    Ok(Json(serde_json::json!({ "domains": domains })))
}

/// POST /api/email/:email_id/domains  { domain_names: ["Sales", "New One"] }
async fn assign_domains(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(email_id): Path<Uuid>,
    Json(req): Json<AssignDomainsRequest>,
) -> Result<Json<AssignResponse>, (StatusCode, String)> {
    let concept_repo = ConceptRepository::new(&app_state.pool);
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let now = Utc::now();

    let mut assigned = Vec::new();
    let mut created = Vec::new();

    for name in &req.domain_names {
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        // Find or create domain
        let concept = match concept_repo
            .find_communication_domain_by_name(&name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            Some(existing) => existing,
            None => {
                let concept_id = Uuid::new_v4();
                let new_concept = Concept {
                    id: concept_id,
                    name: name.clone(),
                    description: None,
                    parent_id: None,
                    keywords: vec![],
                    metadata: serde_json::json!({ "concept_type": "communication_domain" }),
                    created_at: now,
                    updated_at: now,
                    created_by: user.id,
                };
                concept_repo
                    .create(&new_concept)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                created.push(name.clone());
                new_concept
            }
        };

        // Check if link already exists
        let existing_links = cc_repo
            .list_by_communication("email", email_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let already_linked = existing_links.iter().any(|l| l.concept_id == concept.id);
        if !already_linked {
            let cc = CommunicationConcept {
                id: Uuid::new_v4(),
                communication_type: "email".to_string(),
                communication_id: email_id,
                concept_id: concept.id,
                detection_method: DetectionMethod::Manual,
                status: SuggestionStatus::Confirmed,
                confidence: None,
                reviewed_at: Some(now),
                reviewed_by: Some(user.id),
                created_at: now,
            };
            cc_repo
                .create(&cc)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        assigned.push(DomainAssignment {
            concept_id: concept.id,
            name: concept.name,
            status: "confirmed".to_string(),
            detection_method: "manual".to_string(),
        });
    }

    Ok(Json(AssignResponse { assigned, created }))
}

/// DELETE /api/email/:email_id/domains/:concept_id
async fn unlink_domain(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((email_id, concept_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);

    cc_repo
        .delete_by_communication_and_concept("email", email_id, concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/email/bulk-assign-domains { email_ids, domain_names }
async fn bulk_assign_domains(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<BulkAssignRequest>,
) -> Result<Json<BulkAssignResponse>, (StatusCode, String)> {
    let concept_repo = ConceptRepository::new(&app_state.pool);
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let now = Utc::now();

    let mut created_domains = Vec::new();
    let mut total_assigned = 0usize;

    // Resolve all domain names to concepts first
    let mut concepts = Vec::new();
    for name in &req.domain_names {
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        let concept = match concept_repo
            .find_communication_domain_by_name(&name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            Some(existing) => existing,
            None => {
                let concept_id = Uuid::new_v4();
                let new_concept = Concept {
                    id: concept_id,
                    name: name.clone(),
                    description: None,
                    parent_id: None,
                    keywords: vec![],
                    metadata: serde_json::json!({ "concept_type": "communication_domain" }),
                    created_at: now,
                    updated_at: now,
                    created_by: user.id,
                };
                concept_repo
                    .create(&new_concept)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                created_domains.push(name);
                new_concept
            }
        };
        concepts.push(concept);
    }

    // Assign each concept to each email
    for email_id in &req.email_ids {
        let existing_links = cc_repo
            .list_by_communication("email", *email_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        for concept in &concepts {
            let already_linked = existing_links.iter().any(|l| l.concept_id == concept.id);
            if !already_linked {
                let cc = CommunicationConcept {
                    id: Uuid::new_v4(),
                    communication_type: "email".to_string(),
                    communication_id: *email_id,
                    concept_id: concept.id,
                    detection_method: DetectionMethod::Manual,
                    status: SuggestionStatus::Confirmed,
                    confidence: None,
                    reviewed_at: Some(now),
                    reviewed_by: Some(user.id),
                    created_at: now,
                };
                cc_repo
                    .create(&cc)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                total_assigned += 1;
            }
        }
    }

    Ok(Json(BulkAssignResponse {
        assigned_count: total_assigned,
        created_domains,
    }))
}

// --- Sender-level domain endpoints ---

/// GET /api/email/sender/:address/domains
/// Returns deduplicated domains assigned to any email from this sender
async fn get_sender_domains(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(address): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let address = address.to_lowercase();

    // Get all email IDs from this sender
    let email_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM email_history WHERE LOWER(from_address) = ? ORDER BY message_date DESC",
    )
    .bind(&address)
    .fetch_all(app_state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let concept_repo = ConceptRepository::new(&app_state.pool);

    let mut seen_concepts = std::collections::HashSet::new();
    let mut domains = Vec::new();

    for (eid,) in &email_ids {
        if let Ok(uid) = Uuid::parse_str(eid) {
            let links = cc_repo
                .list_by_communication("email", uid)
                .await
                .unwrap_or_default();

            for link in links {
                if seen_concepts.insert(link.concept_id) {
                    if let Ok(concept) = concept_repo.get_by_id(link.concept_id).await {
                        domains.push(DomainAssignment {
                            concept_id: concept.id,
                            name: concept.name,
                            status: link.status.to_string(),
                            detection_method: link.detection_method.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "domains": domains })))
}

/// POST /api/email/sender/:address/domains { domain_names: ["Sales"] }
/// Assigns domains to ALL emails from this sender
async fn assign_domains_to_sender(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(address): Path<String>,
    Json(req): Json<AssignDomainsRequest>,
) -> Result<Json<SenderAssignResponse>, (StatusCode, String)> {
    let address_lower = address.to_lowercase();

    // Get all email IDs from this sender
    let email_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM email_history WHERE LOWER(from_address) = ?",
    )
    .bind(&address_lower)
    .fetch_all(app_state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let email_count = email_ids.len();
    let concept_repo = ConceptRepository::new(&app_state.pool);
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let now = Utc::now();

    let mut created_domains = Vec::new();
    let mut total_assigned = 0usize;

    // Resolve all domain names to concepts
    let mut concepts = Vec::new();
    for name in &req.domain_names {
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        let concept = match concept_repo
            .find_communication_domain_by_name(&name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            Some(existing) => existing,
            None => {
                let concept_id = Uuid::new_v4();
                let new_concept = Concept {
                    id: concept_id,
                    name: name.clone(),
                    description: None,
                    parent_id: None,
                    keywords: vec![],
                    metadata: serde_json::json!({ "concept_type": "communication_domain" }),
                    created_at: now,
                    updated_at: now,
                    created_by: user.id,
                };
                concept_repo
                    .create(&new_concept)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                created_domains.push(name);
                new_concept
            }
        };
        concepts.push(concept);
    }

    // Assign each concept to each email from this sender
    for (eid,) in &email_ids {
        if let Ok(email_uuid) = Uuid::parse_str(eid) {
            let existing_links = cc_repo
                .list_by_communication("email", email_uuid)
                .await
                .unwrap_or_default();

            for concept in &concepts {
                let already_linked = existing_links.iter().any(|l| l.concept_id == concept.id);
                if !already_linked {
                    let cc = CommunicationConcept {
                        id: Uuid::new_v4(),
                        communication_type: "email".to_string(),
                        communication_id: email_uuid,
                        concept_id: concept.id,
                        detection_method: DetectionMethod::Manual,
                        status: SuggestionStatus::Confirmed,
                        confidence: None,
                        reviewed_at: Some(now),
                        reviewed_by: Some(user.id),
                        created_at: now,
                    };
                    let _ = cc_repo.create(&cc).await;
                    total_assigned += 1;
                }
            }
        }
    }

    Ok(Json(SenderAssignResponse {
        assigned_count: total_assigned,
        message_count: email_count,
        created_domains,
    }))
}

/// GET /api/sms/sender/:phone/domains
/// Returns deduplicated domains assigned to any SMS from this phone number
async fn get_sms_sender_domains(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(phone): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Get all SMS IDs from this phone
    let sms_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sms_history WHERE phone_number = ?",
    )
    .bind(&phone)
    .fetch_all(app_state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let concept_repo = ConceptRepository::new(&app_state.pool);

    let mut seen_concepts = std::collections::HashSet::new();
    let mut domains = Vec::new();

    for (sid,) in &sms_ids {
        if let Ok(uid) = Uuid::parse_str(sid) {
            let links = cc_repo
                .list_by_communication("sms", uid)
                .await
                .unwrap_or_default();

            for link in links {
                if seen_concepts.insert(link.concept_id) {
                    if let Ok(concept) = concept_repo.get_by_id(link.concept_id).await {
                        domains.push(DomainAssignment {
                            concept_id: concept.id,
                            name: concept.name,
                            status: link.status.to_string(),
                            detection_method: link.detection_method.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "domains": domains })))
}

/// POST /api/sms/sender/:phone/domains { domain_names: ["Sales"] }
/// Assigns domains to ALL SMS from this phone number
async fn assign_domains_to_sms_sender(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(phone): Path<String>,
    Json(req): Json<AssignDomainsRequest>,
) -> Result<Json<SenderAssignResponse>, (StatusCode, String)> {
    // Get all SMS IDs from this phone
    let sms_ids: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sms_history WHERE phone_number = ?",
    )
    .bind(&phone)
    .fetch_all(app_state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let sms_count = sms_ids.len();
    let concept_repo = ConceptRepository::new(&app_state.pool);
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let now = Utc::now();

    let mut created_domains = Vec::new();
    let mut total_assigned = 0usize;

    // Resolve all domain names to concepts
    let mut concepts = Vec::new();
    for name in &req.domain_names {
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        let concept = match concept_repo
            .find_communication_domain_by_name(&name)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            Some(existing) => existing,
            None => {
                let concept_id = Uuid::new_v4();
                let new_concept = Concept {
                    id: concept_id,
                    name: name.clone(),
                    description: None,
                    parent_id: None,
                    keywords: vec![],
                    metadata: serde_json::json!({ "concept_type": "communication_domain" }),
                    created_at: now,
                    updated_at: now,
                    created_by: user.id,
                };
                concept_repo
                    .create(&new_concept)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                created_domains.push(name);
                new_concept
            }
        };
        concepts.push(concept);
    }

    // Assign each concept to each SMS from this phone
    for (sid,) in &sms_ids {
        if let Ok(sms_uuid) = Uuid::parse_str(sid) {
            let existing_links = cc_repo
                .list_by_communication("sms", sms_uuid)
                .await
                .unwrap_or_default();

            for concept in &concepts {
                let already_linked = existing_links.iter().any(|l| l.concept_id == concept.id);
                if !already_linked {
                    let cc = CommunicationConcept {
                        id: Uuid::new_v4(),
                        communication_type: "sms".to_string(),
                        communication_id: sms_uuid,
                        concept_id: concept.id,
                        detection_method: DetectionMethod::Manual,
                        status: SuggestionStatus::Confirmed,
                        confidence: None,
                        reviewed_at: Some(now),
                        reviewed_by: Some(user.id),
                        created_at: now,
                    };
                    let _ = cc_repo.create(&cc).await;
                    total_assigned += 1;
                }
            }
        }
    }

    Ok(Json(SenderAssignResponse {
        assigned_count: total_assigned,
        message_count: sms_count,
        created_domains,
    }))
}

/// GET /api/sms/:sms_id/domains
/// List domains for a specific SMS message
async fn get_sms_domains(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(sms_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);
    let concept_repo = ConceptRepository::new(&app_state.pool);

    let links = cc_repo
        .list_by_communication("sms", sms_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut domains = Vec::new();
    for link in &links {
        if let Ok(concept) = concept_repo.get_by_id(link.concept_id).await {
            domains.push(DomainAssignment {
                concept_id: concept.id,
                name: concept.name,
                status: link.status.to_string(),
                detection_method: link.detection_method.to_string(),
            });
        }
    }

    Ok(Json(serde_json::json!({ "domains": domains })))
}
