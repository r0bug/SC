use crate::{auth::AuthUser, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use core_domain::{EntityRelationship, EntityType};
use local_store::repositories::{EntityRelationshipRepository, RelationshipTypeRepository};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MatchResult {
    /// The "WANT" relationship (source wants target concept)
    pub want: EntityRelationship,
    /// The "HAVE" relationships that match (other entities have the wanted concept)
    pub haves: Vec<EntityRelationship>,
}

/// Build match routes as a Router
pub fn match_routes() -> Router<AppState> {
    Router::new()
        .route("/api/matches/contact/:id", get(find_contact_matches))
        .route("/api/matches/pick/:id", get(find_pick_matches))
}

/// Helper to filter relationships by type name
fn has_type_name(rel: &EntityRelationship, name: &str) -> bool {
    rel.relationship_type_name
        .as_deref()
        .map(|n| n.eq_ignore_ascii_case(name))
        .unwrap_or(false)
}

/// Helper to get WANT and HAVE type IDs from the database
async fn get_want_have_ids(
    type_repo: &RelationshipTypeRepository<'_>,
) -> Result<(Uuid, Uuid), (StatusCode, &'static str)> {
    let want = type_repo
        .get_by_name("WANT")
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "WANT type not found"))?;
    let have = type_repo
        .get_by_name("HAVE")
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "HAVE type not found"))?;
    Ok((want.id, have.id))
}

/// GET /api/matches/contact/:id - find matches for a specific contact
///
/// Returns concepts that the contact WANTs and other entities that HAVE those concepts.
pub async fn find_contact_matches(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<Vec<MatchResult>>, (StatusCode, &'static str)> {
    let repo = EntityRelationshipRepository::new(&app_state.pool);
    let type_repo = RelationshipTypeRepository::new(&app_state.pool);

    let (want_type_id, _have_type_id) = get_want_have_ids(&type_repo).await?;

    // Get all WANTs for this contact
    let all_rels = repo
        .list_by_source(&EntityType::Contact, contact_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list relationships",
            )
        })?;

    let wants: Vec<_> = all_rels
        .into_iter()
        .filter(|r| r.relationship_type_id == want_type_id)
        .collect();

    // For each WANT, find matching HAVEs on the target concept
    let mut results = Vec::new();
    for want in wants {
        let target_rels = repo
            .list_by_target(&want.target_type, want.target_id)
            .await
            .unwrap_or_default();

        let haves: Vec<_> = target_rels
            .into_iter()
            .filter(|r| has_type_name(r, "HAVE") && r.source_id != contact_id)
            .collect();

        if !haves.is_empty() {
            results.push(MatchResult { want, haves });
        }
    }

    Ok(Json(results))
}

/// GET /api/matches/pick/:id - find contacts that want what a pick has
///
/// Returns concepts that the pick HAS and contacts that WANT those concepts.
pub async fn find_pick_matches(
    State(app_state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(pick_id): Path<Uuid>,
) -> Result<Json<Vec<MatchResult>>, (StatusCode, &'static str)> {
    let repo = EntityRelationshipRepository::new(&app_state.pool);
    let type_repo = RelationshipTypeRepository::new(&app_state.pool);

    let (_want_type_id, have_type_id) = get_want_have_ids(&type_repo).await?;

    // Get all HAVEs for this pick (what it has available)
    let all_rels = repo
        .list_by_source(&EntityType::Pick, pick_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list relationships",
            )
        })?;

    let pick_haves: Vec<_> = all_rels
        .into_iter()
        .filter(|r| r.relationship_type_id == have_type_id)
        .collect();

    // For each HAVE on the pick, find contacts/entities that WANT those concepts
    let mut results = Vec::new();
    for have in &pick_haves {
        let target_rels = repo
            .list_by_target(&have.target_type, have.target_id)
            .await
            .unwrap_or_default();

        let wants: Vec<_> = target_rels
            .into_iter()
            .filter(|r| has_type_name(r, "WANT"))
            .collect();

        for want in wants {
            results.push(MatchResult {
                want,
                haves: vec![have.clone()],
            });
        }
    }

    Ok(Json(results))
}
