use crate::{auth::AuthUser, concept_detection::ConceptDetector, state::AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use core_domain::{ConceptMatcher, ConceptMatcherGroup, MatchMode, MatcherElementType};
use local_store::repositories::{
    CommunicationConceptRepository, ConceptMatcherRepository, ConceptRepository,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request/Response types ---

#[derive(Debug, Deserialize)]
pub struct MatcherInput {
    pub element_type: String,
    pub match_value: String,
    #[serde(default = "default_match_mode")]
    pub match_mode: String,
    #[serde(default)]
    pub negate: bool,
    #[serde(default)]
    pub case_sensitive: bool,
}

fn default_match_mode() -> String {
    "contains".to_string()
}

#[derive(Debug, Deserialize)]
pub struct MatcherGroupInput {
    pub matchers: Vec<MatcherInput>,
}

#[derive(Debug, Deserialize)]
pub struct SetMatchersRequest {
    pub groups: Vec<MatcherGroupInput>,
}

#[derive(Debug, Deserialize)]
pub struct ScanQueryParams {
    pub auto_scan: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LabelQueryParams {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub comm_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MatcherGroupResponse {
    pub id: String,
    pub concept_id: String,
    pub position: i32,
    pub matchers: Vec<MatcherResponse>,
}

#[derive(Debug, Serialize)]
pub struct MatcherResponse {
    pub id: String,
    pub group_id: String,
    pub element_type: String,
    pub match_value: String,
    pub match_mode: String,
    pub negate: bool,
    pub case_sensitive: bool,
    pub position: i32,
}

#[derive(Debug, Serialize)]
pub struct LabelListEntry {
    pub concept_id: String,
    pub name: String,
    pub description: Option<String>,
    pub concept_type: Option<String>,
    pub matcher_count: usize,
    pub confirmed_count: i64,
    pub suggested_count: i64,
}

#[derive(Debug, Serialize)]
pub struct LabelDetail {
    pub concept_id: String,
    pub name: String,
    pub description: Option<String>,
    pub matcher_groups: Vec<MatcherGroupResponse>,
    pub confirmed_count: i64,
    pub suggested_count: i64,
    pub communications: Vec<CommunicationSummary>,
}

#[derive(Debug, Serialize)]
pub struct CommunicationSummary {
    pub id: String,
    pub communication_type: String,
    pub sender: String,
    pub subject: Option<String>,
    pub preview: Option<String>,
    pub date: String,
    pub status: String,
    pub confidence: Option<f32>,
    pub communication_concept_id: String,
}

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub new_suggestions: usize,
    pub total_scanned: usize,
}

// --- Route registration ---

pub fn label_matcher_routes() -> Router<AppState> {
    Router::new()
        // Matcher CRUD for a concept
        .route(
            "/api/concepts/:id/matchers",
            get(get_concept_matchers).post(set_concept_matchers),
        )
        .route("/api/concepts/:id/matchers/groups", post(add_matcher_group))
        .route(
            "/api/concepts/:id/matchers/groups/:gid",
            axum::routing::delete(delete_matcher_group),
        )
        .route(
            "/api/concepts/:id/matchers/groups/:gid/matchers",
            post(add_matcher_to_group),
        )
        .route(
            "/api/concepts/:id/matchers/groups/:gid/matchers/:mid",
            axum::routing::put(update_matcher).delete(delete_matcher),
        )
        // Scanning
        .route("/api/concepts/:id/scan", post(scan_concept))
        .route("/api/labels/scan-all", post(scan_all))
        // Label views
        .route("/api/labels", get(list_labels))
        .route("/api/labels/:id", get(get_label_detail))
}

// --- Handlers ---

/// GET /api/concepts/:id/matchers
async fn get_concept_matchers(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(concept_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);
    let groups = repo
        .list_groups_by_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<MatcherGroupResponse> = groups.into_iter().map(group_to_response).collect();
    Ok(Json(serde_json::json!({ "groups": response })))
}

/// POST /api/concepts/:id/matchers - Replace all matchers (full save)
async fn set_concept_matchers(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(concept_id): Path<Uuid>,
    Query(params): Query<ScanQueryParams>,
    Json(req): Json<SetMatchersRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);
    let now = chrono::Utc::now();

    let groups: Vec<ConceptMatcherGroup> = req
        .groups
        .into_iter()
        .enumerate()
        .map(|(gi, g)| {
            let group_id = Uuid::new_v4();
            ConceptMatcherGroup {
                id: group_id,
                concept_id,
                position: gi as i32,
                matchers: g
                    .matchers
                    .into_iter()
                    .enumerate()
                    .map(|(mi, m)| {
                        Ok(ConceptMatcher {
                            id: Uuid::new_v4(),
                            group_id,
                            element_type: m
                                .element_type
                                .parse::<MatcherElementType>()
                                .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
                            match_value: m.match_value,
                            match_mode: m
                                .match_mode
                                .parse::<MatchMode>()
                                .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
                            negate: m.negate,
                            case_sensitive: m.case_sensitive,
                            position: mi as i32,
                            created_at: now,
                        })
                    })
                    .collect::<Result<Vec<_>, (StatusCode, String)>>()
                    .unwrap_or_default(),
                created_at: now,
                created_by: user.id,
            }
        })
        .collect();

    repo.replace_concept_matchers(concept_id, groups)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Optionally trigger retroactive scan
    let mut scan_count = 0;
    if params.auto_scan.unwrap_or(false) {
        let detector = ConceptDetector::new(&app_state.pool);
        scan_count = detector
            .scan_all_for_concept(concept_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    // Return updated matchers
    let updated_groups = repo
        .list_groups_by_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<MatcherGroupResponse> =
        updated_groups.into_iter().map(group_to_response).collect();

    Ok(Json(serde_json::json!({
        "groups": response,
        "scan_count": scan_count
    })))
}

/// POST /api/concepts/:id/matchers/groups - Add a single group
async fn add_matcher_group(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(concept_id): Path<Uuid>,
    Json(req): Json<MatcherGroupInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);
    let now = chrono::Utc::now();

    // Find next position
    let existing = repo
        .list_groups_by_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let next_pos = existing.iter().map(|g| g.position).max().unwrap_or(-1) + 1;

    let group_id = Uuid::new_v4();
    let group = ConceptMatcherGroup {
        id: group_id,
        concept_id,
        position: next_pos,
        matchers: req
            .matchers
            .into_iter()
            .enumerate()
            .map(|(mi, m)| {
                Ok(ConceptMatcher {
                    id: Uuid::new_v4(),
                    group_id,
                    element_type: m
                        .element_type
                        .parse::<MatcherElementType>()
                        .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
                    match_value: m.match_value,
                    match_mode: m
                        .match_mode
                        .parse::<MatchMode>()
                        .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
                    negate: m.negate,
                    case_sensitive: m.case_sensitive,
                    position: mi as i32,
                    created_at: now,
                })
            })
            .collect::<Result<Vec<_>, (StatusCode, String)>>()?,
        created_at: now,
        created_by: user.id,
    };

    repo.create_group(&group)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for matcher in &group.matchers {
        repo.create_matcher(matcher)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(
        serde_json::json!({ "group": group_to_response(group) }),
    ))
}

/// DELETE /api/concepts/:id/matchers/groups/:gid
async fn delete_matcher_group(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((_concept_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);
    repo.delete_group(group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/concepts/:id/matchers/groups/:gid/matchers - Add matcher to group
async fn add_matcher_to_group(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((_concept_id, group_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<MatcherInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);
    let now = chrono::Utc::now();

    // Find next position
    let existing = repo
        .list_matchers_by_group(group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let next_pos = existing.iter().map(|m| m.position).max().unwrap_or(-1) + 1;

    let matcher = ConceptMatcher {
        id: Uuid::new_v4(),
        group_id,
        element_type: req
            .element_type
            .parse::<MatcherElementType>()
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
        match_value: req.match_value,
        match_mode: req
            .match_mode
            .parse::<MatchMode>()
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
        negate: req.negate,
        case_sensitive: req.case_sensitive,
        position: next_pos,
        created_at: now,
    };

    repo.create_matcher(&matcher)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "matcher": matcher_to_response(&matcher) }),
    ))
}

/// PUT /api/concepts/:id/matchers/groups/:gid/matchers/:mid
async fn update_matcher(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((_concept_id, group_id, matcher_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(req): Json<MatcherInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);

    let matcher = ConceptMatcher {
        id: matcher_id,
        group_id,
        element_type: req
            .element_type
            .parse::<MatcherElementType>()
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
        match_value: req.match_value,
        match_mode: req
            .match_mode
            .parse::<MatchMode>()
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?,
        negate: req.negate,
        case_sensitive: req.case_sensitive,
        position: 0, // position preserved on update
        created_at: chrono::Utc::now(),
    };

    repo.update_matcher(&matcher)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "matcher": matcher_to_response(&matcher) }),
    ))
}

/// DELETE /api/concepts/:id/matchers/groups/:gid/matchers/:mid
async fn delete_matcher(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path((_concept_id, _group_id, matcher_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let repo = ConceptMatcherRepository::new(&app_state.pool);
    repo.delete_matcher(matcher_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/concepts/:id/scan - Retroactive scan for one concept
async fn scan_concept(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(concept_id): Path<Uuid>,
) -> Result<Json<ScanResult>, (StatusCode, String)> {
    let detector = ConceptDetector::new(&app_state.pool);
    let new_suggestions = detector
        .scan_all_for_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ScanResult {
        new_suggestions,
        total_scanned: 0, // could count emails+sms but not critical
    }))
}

/// POST /api/labels/scan-all - Scan all concepts against all communications
async fn scan_all(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> Result<Json<ScanResult>, (StatusCode, String)> {
    let detector = ConceptDetector::new(&app_state.pool);
    let new_suggestions = detector
        .scan_all()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ScanResult {
        new_suggestions,
        total_scanned: 0,
    }))
}

/// GET /api/labels - List all labels with match counts
async fn list_labels(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let concept_repo = ConceptRepository::new(&app_state.pool);
    let matcher_repo = ConceptMatcherRepository::new(&app_state.pool);
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);

    let concepts = concept_repo
        .list(10000, 0)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let concepts_with_matchers = matcher_repo
        .load_all_concepts_with_matchers()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let matcher_concept_ids: std::collections::HashSet<Uuid> = concepts_with_matchers
        .iter()
        .map(|c| c.concept_id)
        .collect();

    let mut labels: Vec<LabelListEntry> = Vec::new();

    for concept in &concepts {
        // Include concepts that have matchers OR have any communication_concept links
        let cc_list = cc_repo
            .list_by_concept(concept.id)
            .await
            .unwrap_or_default();

        let has_matchers = matcher_concept_ids.contains(&concept.id);
        if !has_matchers && cc_list.is_empty() {
            continue;
        }

        let matcher_count = concepts_with_matchers
            .iter()
            .find(|c| c.concept_id == concept.id)
            .map(|c| c.groups.iter().map(|g| g.matchers.len()).sum::<usize>())
            .unwrap_or(0);

        let confirmed_count = cc_list
            .iter()
            .filter(|c| c.status.to_string() == "confirmed")
            .count() as i64;
        let suggested_count = cc_list
            .iter()
            .filter(|c| c.status.to_string() == "suggested")
            .count() as i64;

        // Determine concept_type from metadata
        let concept_type = concept
            .metadata
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        labels.push(LabelListEntry {
            concept_id: concept.id.to_string(),
            name: concept.name.clone(),
            description: concept.description.clone(),
            concept_type,
            matcher_count,
            confirmed_count,
            suggested_count,
        });
    }

    Ok(Json(serde_json::json!({ "labels": labels })))
}

/// GET /api/labels/:id - Label detail: concept + criteria + confirmed/suggested matches
async fn get_label_detail(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(concept_id): Path<Uuid>,
    Query(params): Query<LabelQueryParams>,
) -> Result<Json<LabelDetail>, (StatusCode, String)> {
    let concept_repo = ConceptRepository::new(&app_state.pool);
    let matcher_repo = ConceptMatcherRepository::new(&app_state.pool);
    let cc_repo = CommunicationConceptRepository::new(&app_state.pool);

    let concept = concept_repo
        .get_by_id(concept_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let groups = matcher_repo
        .list_groups_by_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let all_ccs = cc_repo
        .list_by_concept(concept_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let status_filter = params.status.as_deref().unwrap_or("all");
    let comm_type_filter = params.comm_type.as_deref().unwrap_or("all");
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let confirmed_count = all_ccs
        .iter()
        .filter(|c| c.status.to_string() == "confirmed")
        .count() as i64;
    let suggested_count = all_ccs
        .iter()
        .filter(|c| c.status.to_string() == "suggested")
        .count() as i64;

    // Filter by status and comm_type
    let filtered_ccs: Vec<_> = all_ccs
        .iter()
        .filter(|cc| {
            let status_ok = match status_filter {
                "confirmed" => cc.status.to_string() == "confirmed",
                "suggested" => cc.status.to_string() == "suggested",
                _ => cc.status.to_string() != "denied",
            };
            let type_ok = match comm_type_filter {
                "email" => cc.communication_type == "email",
                "sms" => cc.communication_type == "sms",
                _ => true,
            };
            status_ok && type_ok
        })
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    // Build communication summaries
    let email_repo = local_store::repositories::EmailHistoryRepository::new(&app_state.pool);
    let sms_repo = local_store::repositories::SmsHistoryRepository::new(&app_state.pool);

    let mut communications = Vec::new();
    for cc in &filtered_ccs {
        let summary = match cc.communication_type.as_str() {
            "email" => {
                let email = get_email_by_id(&email_repo, cc.communication_id).await;
                match email {
                    Some(e) => CommunicationSummary {
                        id: e.id.to_string(),
                        communication_type: "email".to_string(),
                        sender: e.from_address.clone(),
                        subject: Some(e.subject.clone()),
                        preview: e.body_text.as_ref().map(|t| t.chars().take(100).collect()),
                        date: chrono::DateTime::from_timestamp_millis(e.message_date)
                            .map(|d| d.to_rfc3339())
                            .unwrap_or_default(),
                        status: cc.status.to_string(),
                        confidence: cc.confidence,
                        communication_concept_id: cc.id.to_string(),
                    },
                    None => continue,
                }
            }
            "sms" => {
                let sms = get_sms_by_id(&sms_repo, cc.communication_id).await;
                match sms {
                    Some(s) => CommunicationSummary {
                        id: s.id.to_string(),
                        communication_type: "sms".to_string(),
                        sender: s.phone_number.clone(),
                        subject: s.subject.clone(),
                        preview: Some(s.body.chars().take(100).collect()),
                        date: chrono::DateTime::from_timestamp_millis(s.message_date)
                            .map(|d| d.to_rfc3339())
                            .unwrap_or_default(),
                        status: cc.status.to_string(),
                        confidence: cc.confidence,
                        communication_concept_id: cc.id.to_string(),
                    },
                    None => continue,
                }
            }
            _ => continue,
        };
        communications.push(summary);
    }

    let matcher_groups: Vec<MatcherGroupResponse> =
        groups.into_iter().map(group_to_response).collect();

    Ok(Json(LabelDetail {
        concept_id: concept.id.to_string(),
        name: concept.name.clone(),
        description: concept.description.clone(),
        matcher_groups,
        confirmed_count,
        suggested_count,
        communications,
    }))
}

// --- Helper functions ---

fn group_to_response(g: ConceptMatcherGroup) -> MatcherGroupResponse {
    MatcherGroupResponse {
        id: g.id.to_string(),
        concept_id: g.concept_id.to_string(),
        position: g.position,
        matchers: g.matchers.iter().map(matcher_to_response).collect(),
    }
}

fn matcher_to_response(m: &ConceptMatcher) -> MatcherResponse {
    MatcherResponse {
        id: m.id.to_string(),
        group_id: m.group_id.to_string(),
        element_type: m.element_type.to_string(),
        match_value: m.match_value.clone(),
        match_mode: m.match_mode.to_string(),
        negate: m.negate,
        case_sensitive: m.case_sensitive,
        position: m.position,
    }
}

/// Look up a single email by ID (using get_by_address workaround via list_all)
async fn get_email_by_id(
    repo: &local_store::repositories::EmailHistoryRepository<'_>,
    id: Uuid,
) -> Option<local_store::repositories::EmailMessage> {
    // Use a simple query approach - get_block with large limit is expensive
    // Instead, reuse the existing concept-based query
    let all = repo.get_block(0, 10000).await.ok()?;
    all.into_iter().find(|e| e.id == id)
}

/// Look up a single SMS by ID
async fn get_sms_by_id(
    repo: &local_store::repositories::SmsHistoryRepository<'_>,
    id: Uuid,
) -> Option<local_store::repositories::SmsMessage> {
    let all = repo.list_all().await.ok()?;
    all.into_iter().find(|s| s.id == id)
}
