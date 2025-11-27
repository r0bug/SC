use crate::audit;
use crate::auth::AuthUser;
use crate::state::AppState;
use crate::validation;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use core_domain::{Attachment, AttachmentEntityType, Permission, ShareEntityType};
use local_store::{
    AttachmentRepository, CalendarEventRepository, CommunicationRepository, ContactRepository,
    NoteRepository, ProjectRepository,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListAttachmentsQuery {
    pub entity_type: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub attachment: Attachment,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Helper function to convert AttachmentEntityType to ShareEntityType for audit logging
fn attachment_entity_to_share_entity(entity_type: AttachmentEntityType) -> ShareEntityType {
    match entity_type {
        AttachmentEntityType::Contact => ShareEntityType::Contact,
        AttachmentEntityType::Project => ShareEntityType::Project,
        AttachmentEntityType::Note => ShareEntityType::Note,
        AttachmentEntityType::CalendarEvent => ShareEntityType::CalendarEvent,
        // Map Communication to Contact as a workaround (Communication not in ShareEntityType)
        AttachmentEntityType::Communication => ShareEntityType::Contact,
    }
}

async fn resolve_attachment_owner(
    state: &AppState,
    entity_type: AttachmentEntityType,
    entity_id: Uuid,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    match entity_type {
        AttachmentEntityType::Contact => {
            let repo = ContactRepository::new(state.store.pool());
            let contact = repo.get_by_id(entity_id).await.map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Contact not found".to_string(),
                    }),
                )
            })?;
            Ok(contact.created_by)
        }
        AttachmentEntityType::Project => {
            let repo = ProjectRepository::new(state.store.pool());
            let project = repo.get_by_id(entity_id).await.map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Project not found".to_string(),
                    }),
                )
            })?;
            Ok(project.created_by)
        }
        AttachmentEntityType::Note => {
            let repo = NoteRepository::new(state.store.pool());
            let note = repo.get_by_id(entity_id).await.map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Note not found".to_string(),
                    }),
                )
            })?;
            Ok(note.created_by)
        }
        AttachmentEntityType::CalendarEvent => {
            let repo = CalendarEventRepository::new(state.store.pool());
            let event = repo.get_by_id(entity_id).await.map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Calendar event not found".to_string(),
                    }),
                )
            })?;
            Ok(event.created_by)
        }
        AttachmentEntityType::Communication => {
            let comm_repo = CommunicationRepository::new(state.store.pool());
            let communication = comm_repo.get_communication(entity_id).await.map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Communication not found".to_string(),
                    }),
                )
            })?;
            let contact_repo = ContactRepository::new(state.store.pool());
            let contact = contact_repo
                .get_by_id(communication.contact_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse {
                            error: "Contact not found".to_string(),
                        }),
                    )
                })?;
            Ok(contact.created_by)
        }
    }
}

async fn ensure_attachment_permission(
    state: &AppState,
    user_id: &Uuid,
    entity_type: AttachmentEntityType,
    entity_id: Uuid,
    permission: Permission,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let owner_id = resolve_attachment_owner(state, entity_type.clone(), entity_id).await?;

    let permitted = match permission {
        Permission::Read => {
            state
                .acl_service
                .can_read(
                    user_id,
                    attachment_entity_to_share_entity(entity_type.clone()),
                    &entity_id,
                )
                .await
        }
        Permission::Write => {
            state
                .acl_service
                .can_write(
                    user_id,
                    attachment_entity_to_share_entity(entity_type.clone()),
                    &entity_id,
                )
                .await
        }
        Permission::Delete => {
            state
                .acl_service
                .can_delete(
                    user_id,
                    attachment_entity_to_share_entity(entity_type.clone()),
                    &entity_id,
                )
                .await
        }
        Permission::Share => {
            state
                .acl_service
                .can_share(
                    user_id,
                    attachment_entity_to_share_entity(entity_type.clone()),
                    &entity_id,
                )
                .await
        }
    };

    match permitted {
        Ok(true) => Ok(()),
        Ok(false) => {
            if owner_id == *user_id {
                Ok(())
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Access denied for attachment operation".to_string(),
                    }),
                ))
            }
        }
        Err(err) => Err((
            StatusCode::from(err),
            Json(ErrorResponse {
                error: "Failed to verify attachment permissions".to_string(),
            }),
        )),
    }
}

/// Upload a new attachment
/// POST /api/attachments/upload
/// Content-Type: multipart/form-data
/// Fields: file, entity_type, entity_id
pub async fn upload_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut entity_type: Option<String> = None;
    let mut entity_id: Option<Uuid> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid multipart data".to_string(),
            }),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| {
                            tracing::error!("Failed to read file data: {}", e);
                            (
                                StatusCode::BAD_REQUEST,
                                Json(ErrorResponse {
                                    error: "Failed to read file".to_string(),
                                }),
                            )
                        })?
                        .to_vec(),
                );
            }
            "entity_type" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!("Failed to read entity_type: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Invalid entity_type".to_string(),
                        }),
                    )
                })?;
                entity_type = Some(text);
            }
            "entity_id" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!("Failed to read entity_id: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Invalid entity_id".to_string(),
                        }),
                    )
                })?;
                entity_id = Some(Uuid::parse_str(&text).map_err(|e| {
                    tracing::error!("Failed to parse entity_id UUID: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Invalid entity_id format".to_string(),
                        }),
                    )
                })?);
            }
            _ => {
                tracing::warn!("Unknown multipart field: {}", field_name);
            }
        }
    }

    // Validate required fields
    let file_data = file_data.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing file".to_string(),
            }),
        )
    })?;

    let filename = filename.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing filename".to_string(),
            }),
        )
    })?;

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    // Comprehensive file upload validation (size, extension, MIME type, path safety)
    validation::validate_file_upload(&filename, &content_type, file_data.len())
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e.0 })))?;

    // Sanitize filename to prevent special character issues
    let safe_filename = validation::sanitize_filename(&filename);

    let entity_type_str = entity_type.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing entity_type".to_string(),
            }),
        )
    })?;

    let entity_id = entity_id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing entity_id".to_string(),
            }),
        )
    })?;

    // Use authenticated user ID instead of client-supplied uploaded_by
    let uploaded_by = user.id;

    // Parse entity type
    let entity_type = match entity_type_str.as_str() {
        "Contact" => AttachmentEntityType::Contact,
        "Note" => AttachmentEntityType::Note,
        "Project" => AttachmentEntityType::Project,
        "CalendarEvent" => AttachmentEntityType::CalendarEvent,
        "Communication" => AttachmentEntityType::Communication,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid entity_type: {}", entity_type_str),
                }),
            ));
        }
    };

    ensure_attachment_permission(
        &state,
        &user.id,
        entity_type.clone(),
        entity_id,
        Permission::Write,
    )
    .await?;

    // Calculate checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let checksum = hex::encode(hasher.finalize());

    // Clone entity_type for audit logging before the original is moved into the Attachment struct
    let entity_type_for_audit = entity_type.clone();

    // For now, store locally in data/attachments
    // TODO: Use AttachmentService with proper storage backend
    let storage_dir = std::path::PathBuf::from("./data/attachments");
    tokio::fs::create_dir_all(&storage_dir).await.map_err(|e| {
        tracing::error!("Failed to create storage directory: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create storage directory".to_string(),
            }),
        )
    })?;

    // Generate unique storage path
    let file_id = Uuid::new_v4();
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let storage_filename = format!("{}.{}", file_id, extension);
    let storage_path = storage_dir.join(&storage_filename);

    tokio::fs::write(&storage_path, &file_data)
        .await
        .map_err(|e| {
            tracing::error!("Failed to write file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to save file".to_string(),
                }),
            )
        })?;

    // Scan file for viruses
    let scan_result = state
        .virus_scanner
        .scan_file(&storage_path)
        .await
        .map_err(|e| {
            tracing::error!("Virus scan error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Virus scan error: {}", e),
                }),
            )
        })?;

    let (scan_status, scan_details) = match scan_result {
        crate::virus_scanner::ScanResult::Clean => (core_domain::ScanStatus::Clean, None),
        crate::virus_scanner::ScanResult::Infected(virus_name) => {
            // Delete infected file immediately
            let _ = tokio::fs::remove_file(&storage_path).await;
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("File infected with: {}", virus_name),
                }),
            ));
        }
        crate::virus_scanner::ScanResult::Error(msg) => (core_domain::ScanStatus::Error, Some(msg)),
    };

    // Create attachment record with sanitized filename
    let attachment = Attachment {
        id: Uuid::new_v4(),
        filename: safe_filename,
        content_type,
        size_bytes: file_data.len() as i64,
        storage_path: storage_path.to_string_lossy().to_string(),
        thumbnail_path: None,
        entity_type,
        entity_id,
        uploaded_by,
        checksum,
        encrypted: false,
        scan_status,
        scan_details,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    // Save to database
    let repo = AttachmentRepository::new(state.store.pool());
    repo.create(&attachment).await.map_err(|e| {
        tracing::error!("Failed to create attachment record: {}", e);
        // Clean up file
        let _ = std::fs::remove_file(&storage_path);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to save attachment".to_string(),
            }),
        )
    })?;

    // Audit log: attachment uploaded
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let changes = serde_json::json!({
        "filename": attachment.filename,
        "content_type": attachment.content_type,
        "size_bytes": attachment.size_bytes,
        "entity_type": entity_type_str,
        "entity_id": entity_id
    });
    let share_entity_type = attachment_entity_to_share_entity(entity_type_for_audit);
    let _ = state
        .audit_service
        .log_operation(
            share_entity_type,
            attachment.entity_id,
            core_domain::AuditAction::Create,
            user.id,
            changes,
            ip,
            user_agent,
        )
        .await;

    tracing::info!(
        "Uploaded attachment: {} ({} bytes)",
        attachment.id,
        attachment.size_bytes
    );

    Ok(Json(UploadResponse { attachment }))
}

/// List attachments for an entity
/// GET /api/attachments?entity_type={type}&entity_id={id}
pub async fn list_attachments(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListAttachmentsQuery>,
) -> Result<Json<Vec<Attachment>>, (StatusCode, Json<ErrorResponse>)> {
    let entity_type = match params.entity_type.as_str() {
        "Contact" => AttachmentEntityType::Contact,
        "Note" => AttachmentEntityType::Note,
        "Project" => AttachmentEntityType::Project,
        "CalendarEvent" => AttachmentEntityType::CalendarEvent,
        "Communication" => AttachmentEntityType::Communication,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid entity_type: {}", params.entity_type),
                }),
            ));
        }
    };

    ensure_attachment_permission(
        &state,
        &user.id,
        entity_type.clone(),
        params.entity_id,
        Permission::Read,
    )
    .await?;

    let repo = AttachmentRepository::new(state.store.pool());
    let attachments = repo
        .list_by_entity(entity_type, params.entity_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list attachments: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to list attachments".to_string(),
                }),
            )
        })?;

    Ok(Json(attachments))
}

/// Download an attachment
/// GET /api/attachments/{id}
pub async fn download_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let repo = AttachmentRepository::new(state.store.pool());
    let attachment = repo.get_by_id(id).await.map_err(|e| {
        tracing::error!("Attachment not found: {}", e);
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Attachment not found".to_string(),
            }),
        )
    })?;

    // Check if user has permission to read attachments on the parent entity
    ensure_attachment_permission(
        &state,
        &user.id,
        attachment.entity_type.clone(),
        attachment.entity_id,
        Permission::Read,
    )
    .await?;

    // Check scan status
    if attachment.scan_status == core_domain::ScanStatus::Infected {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: format!(
                    "File is infected: {}",
                    attachment.scan_details.unwrap_or_default()
                ),
            }),
        ));
    }

    // Read file
    let file_data = tokio::fs::read(&attachment.storage_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to read file".to_string(),
                }),
            )
        })?;

    // Verify checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let checksum = hex::encode(hasher.finalize());

    if checksum != attachment.checksum {
        tracing::error!(
            "Checksum mismatch for attachment {}: expected {}, got {}",
            id,
            attachment.checksum,
            checksum
        );
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "File integrity check failed".to_string(),
            }),
        ));
    }

    // Audit log: attachment downloaded (read access)
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    // Save values before partial move
    let attachment_entity_type = attachment.entity_type;
    let attachment_entity_id = attachment.entity_id;
    let changes = serde_json::json!({
        "filename": attachment.filename,
        "action": "download"
    });
    let share_entity_type = attachment_entity_to_share_entity(attachment_entity_type);
    let _ = state
        .audit_service
        .log_operation(
            share_entity_type,
            attachment_entity_id,
            core_domain::AuditAction::Read,
            user.id,
            changes,
            ip,
            user_agent,
        )
        .await;

    // Return file with appropriate headers
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, attachment.content_type.clone())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", attachment.filename),
        )
        .header(header::CONTENT_LENGTH, attachment.size_bytes)
        .body(Body::from(file_data))
        .unwrap())
}

/// Delete an attachment
/// DELETE /api/attachments/{id}
pub async fn delete_attachment(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let repo = AttachmentRepository::new(state.store.pool());

    // Get attachment to find file path
    let attachment = repo.get_by_id(id).await.map_err(|e| {
        tracing::error!("Attachment not found: {}", e);
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Attachment not found".to_string(),
            }),
        )
    })?;

    // Check if user has permission to delete attachments on the parent entity
    ensure_attachment_permission(
        &state,
        &user.id,
        attachment.entity_type.clone(),
        attachment.entity_id,
        Permission::Delete,
    )
    .await?;

    // Save attachment info for audit log before deletion
    let attachment_filename = attachment.filename.clone();
    let attachment_entity_type = attachment.entity_type;
    let attachment_entity_id = attachment.entity_id;

    // Delete file from storage
    if let Err(e) = tokio::fs::remove_file(&attachment.storage_path).await {
        tracing::warn!("Failed to delete file {}: {}", attachment.storage_path, e);
        // Continue anyway - database record is more important
    }

    // Delete from database
    repo.delete(id).await.map_err(|e| {
        tracing::error!("Failed to delete attachment record: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to delete attachment".to_string(),
            }),
        )
    })?;

    // Audit log: attachment deleted
    let ip = audit::extract_ip_address(&headers);
    let user_agent = audit::extract_user_agent(&headers);
    let changes = serde_json::json!({
        "filename": attachment_filename,
        "deleted": true
    });
    let share_entity_type = attachment_entity_to_share_entity(attachment_entity_type);
    let _ = state
        .audit_service
        .log_operation(
            share_entity_type,
            attachment_entity_id,
            core_domain::AuditAction::Delete,
            user.id,
            changes,
            ip,
            user_agent,
        )
        .await;

    tracing::info!("Deleted attachment: {}", id);

    Ok(StatusCode::NO_CONTENT)
}
