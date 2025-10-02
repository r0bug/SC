use crate::{auth::AuthUser, state::AppState, validation};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use core_domain::{CalendarEvent, Reminder};
use local_store::repositories::CalendarEventRepository;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub contacts: Vec<Uuid>,
    pub location: Option<String>,
    pub reminders: Vec<Reminder>,
    pub attachment_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EventQueryParams {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// POST /api/calendar/events
pub async fn create_event(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateEventRequest>,
) -> Result<Json<CalendarEvent>, (StatusCode, String)> {
    // Validate inputs
    validation::validate_title(&req.title)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_optional(&req.description, |d: &String| validation::validate_description(d))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_optional(&req.location, |l: &String| validation::validate_location(l))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(&req.contacts, validation::MAX_CONTACTS_COUNT, "contacts")
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;
    validation::validate_uuid_list(&req.attachment_ids, validation::MAX_CONTACTS_COUNT, "attachments")
        .map_err(|e| (StatusCode::BAD_REQUEST, e.0))?;

    // Validate that end_time is after start_time
    if req.end_time <= req.start_time {
        return Err((StatusCode::BAD_REQUEST, "End time must be after start time".to_string()));
    }

    let event = CalendarEvent {
        id: Uuid::new_v4(),
        title: req.title,
        description: req.description,
        start_time: req.start_time,
        end_time: req.end_time,
        all_day: req.all_day,
        contacts: req.contacts,
        location: req.location,
        recurrence: None,
        reminders: req.reminders,
        attachment_ids: req.attachment_ids,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user.id,
        version: 1,
    };

    let repo = CalendarEventRepository::new(&app_state.pool);
    repo.create(&event)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create event".to_string()))?;

    // Create ACL for the event
    app_state
        .acl_service
        .create_acl(
            &user.id,
            core_domain::ShareEntityType::CalendarEvent,
            &event.id,
        )
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create ACL".to_string()))?;

    Ok(Json(event))
}

/// GET /api/calendar/events/:id
pub async fn get_event(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Check permission
    if !app_state
        .acl_service
        .can_read(&user.id, core_domain::ShareEntityType::CalendarEvent, &id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = CalendarEventRepository::new(&app_state.pool);
    let event = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Event not found"))?;

    Ok(Json(event))
}

/// PUT /api/calendar/events/:id
pub async fn update_event(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    // Check permission
    if !app_state
        .acl_service
        .can_write(&user.id, core_domain::ShareEntityType::CalendarEvent, &id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = CalendarEventRepository::new(&app_state.pool);
    let mut event = repo
        .get_by_id(id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Event not found"))?;

    event.title = req.title;
    event.description = req.description;
    event.start_time = req.start_time;
    event.end_time = req.end_time;
    event.all_day = req.all_day;
    event.contacts = req.contacts;
    event.location = req.location;
    event.reminders = req.reminders;
    event.attachment_ids = req.attachment_ids;
    event.updated_at = chrono::Utc::now();
    event.version += 1;

    repo.update(&event)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update event"))?;

    Ok(Json(event))
}

/// DELETE /api/calendar/events/:id
pub async fn delete_event(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Check permission
    if !app_state
        .acl_service
        .can_delete(&user.id, core_domain::ShareEntityType::CalendarEvent, &id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = CalendarEventRepository::new(&app_state.pool);
    repo.delete(id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete event"))?;

    Ok(Json(serde_json::json!({ "message": "Event deleted" })))
}

/// GET /api/calendar/events
pub async fn list_events(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<Vec<CalendarEvent>>, (StatusCode, &'static str)> {
    let repo = CalendarEventRepository::new(&app_state.pool);

    let events = if let (Some(start), Some(end)) = (params.start, params.end) {
        repo.list_by_date_range(user.id, start, end).await
    } else {
        repo.list_by_creator(user.id).await
    }
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list events"))?;

    Ok(Json(events))
}

/// GET /api/calendar/events/contact/:contact_id
pub async fn list_events_by_contact(
    State(app_state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<Vec<CalendarEvent>>, (StatusCode, &'static str)> {
    // Check if user has access to the contact
    if !app_state
        .acl_service
        .can_read(&user.id, core_domain::ShareEntityType::Contact, &contact_id)
        .await
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let repo = CalendarEventRepository::new(&app_state.pool);
    let events = repo
        .list_by_contact(contact_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list events"))?;

    Ok(Json(events))
}