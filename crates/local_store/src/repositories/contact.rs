use core_domain::{Contact, DomainError, DomainResult, SocialHandle};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct ContactRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> ContactRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, contact: &Contact) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO contacts (id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(contact.id.to_string())
        .bind(&contact.first_name)
        .bind(&contact.last_name)
        .bind(&contact.email)
        .bind(&contact.phone)
        .bind(&contact.organization)
        .bind(&contact.title)
        .bind(&contact.notes)
        .bind(serde_json::to_string(&contact.metadata).unwrap())
        .bind(contact.created_at.to_rfc3339())
        .bind(contact.updated_at.to_rfc3339())
        .bind(contact.created_by.to_string())
        .bind(contact.version)
        .bind(contact.last_synced_at.map(|d| d.to_rfc3339()))
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for handle in &contact.social_handles {
            sqlx::query(
                "INSERT INTO social_handles (contact_id, platform, handle, url) VALUES (?, ?, ?, ?)"
            )
            .bind(contact.id.to_string())
            .bind(&handle.platform)
            .bind(&handle.handle)
            .bind(&handle.url)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_phone(&self, phone: &str) -> DomainResult<Option<Contact>> {
        let row = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts WHERE phone = ? LIMIT 1"
        )
        .bind(phone)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Some(row) = row {
            let social_handles = sqlx::query_as::<_, SocialHandleRow>(
                "SELECT platform, handle, url FROM social_handles WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            let tags = sqlx::query_scalar::<_, String>(
                "SELECT tag_id FROM contact_tags WHERE contact_id = ?",
            )
            .bind(&row.id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let projects = sqlx::query_scalar::<_, String>(
                "SELECT project_id FROM project_contacts WHERE contact_id = ?",
            )
            .bind(&row.id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let groups = sqlx::query_scalar::<_, String>(
                "SELECT group_id FROM contact_groups WHERE contact_id = ?",
            )
            .bind(&row.id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            Ok(Some(row.into_contact(
                social_handles,
                tags,
                projects,
                groups,
            )))
        } else {
            Ok(None)
        }
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Contact> {
        let row = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Contact {}", id)))?;

        let social_handles = sqlx::query_as::<_, SocialHandleRow>(
            "SELECT platform, handle, url FROM social_handles WHERE contact_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let tags =
            sqlx::query_scalar::<_, String>("SELECT tag_id FROM contact_tags WHERE contact_id = ?")
                .bind(id.to_string())
                .fetch_all(self.pool)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?
                .into_iter()
                .filter_map(|s| Uuid::parse_str(&s).ok())
                .collect();

        let projects = sqlx::query_scalar::<_, String>(
            "SELECT project_id FROM project_contacts WHERE contact_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        let groups = sqlx::query_scalar::<_, String>(
            "SELECT group_id FROM contact_groups WHERE contact_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        Ok(row.into_contact(social_handles, tags, projects, groups))
    }

    pub async fn list(&self, limit: i64, offset: i64, user_id: Uuid) -> DomainResult<Vec<Contact>> {
        let rows = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts WHERE created_by = ? ORDER BY last_name, first_name LIMIT ? OFFSET ?"
        )
        .bind(user_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut contacts = Vec::new();
        for row in rows {
            let social_handles = sqlx::query_as::<_, SocialHandleRow>(
                "SELECT platform, handle, url FROM social_handles WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            let tags = sqlx::query_scalar::<_, String>(
                "SELECT tag_id FROM contact_tags WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let projects = sqlx::query_scalar::<_, String>(
                "SELECT project_id FROM project_contacts WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let groups = sqlx::query_scalar::<_, String>(
                "SELECT group_id FROM contact_groups WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            contacts.push(row.into_contact(social_handles, tags, projects, groups));
        }

        Ok(contacts)
    }

    pub async fn update(&self, contact: &Contact) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE contacts SET first_name = ?, last_name = ?, email = ?, phone = ?,
             organization = ?, title = ?, notes = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&contact.first_name)
        .bind(&contact.last_name)
        .bind(&contact.email)
        .bind(&contact.phone)
        .bind(&contact.organization)
        .bind(&contact.title)
        .bind(&contact.notes)
        .bind(serde_json::to_string(&contact.metadata).unwrap())
        .bind(contact.updated_at.to_rfc3339())
        .bind(contact.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM social_handles WHERE contact_id = ?")
            .bind(contact.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for handle in &contact.social_handles {
            sqlx::query(
                "INSERT INTO social_handles (contact_id, platform, handle, url) VALUES (?, ?, ?, ?)"
            )
            .bind(contact.id.to_string())
            .bind(&handle.platform)
            .bind(&handle.handle)
            .bind(&handle.url)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn search(&self, query: &str, user_id: Uuid) -> DomainResult<Vec<Contact>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts
             WHERE created_by = ? AND (first_name LIKE ? OR last_name LIKE ? OR email LIKE ? OR phone LIKE ? OR organization LIKE ?)
             ORDER BY last_name, first_name"
        )
        .bind(user_id.to_string())
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut contacts = Vec::new();
        for row in rows {
            let social_handles = sqlx::query_as::<_, SocialHandleRow>(
                "SELECT platform, handle, url FROM social_handles WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            let tags = sqlx::query_scalar::<_, String>(
                "SELECT tag_id FROM contact_tags WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let projects = sqlx::query_scalar::<_, String>(
                "SELECT project_id FROM project_contacts WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let groups = sqlx::query_scalar::<_, String>(
                "SELECT group_id FROM contact_groups WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            contacts.push(row.into_contact(social_handles, tags, projects, groups));
        }

        Ok(contacts)
    }

    /// Find duplicate contacts grouped by phone number
    /// Returns a map of phone numbers to lists of contacts with that phone number
    /// Only includes phone numbers with 2+ contacts
    pub async fn find_duplicates_by_phone(
        &self,
    ) -> DomainResult<std::collections::HashMap<String, Vec<Contact>>> {
        use std::collections::HashMap;

        // Get all contacts with phone numbers
        let rows = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts
             WHERE phone IS NOT NULL AND phone != ''
             ORDER BY phone, created_at"
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut duplicates: HashMap<String, Vec<Contact>> = HashMap::new();

        for row in rows {
            // Clone phone before borrowing row
            let phone = match &row.phone {
                Some(p) => p.clone(),
                None => continue,
            };

            let social_handles = sqlx::query_as::<_, SocialHandleRow>(
                "SELECT platform, handle, url FROM social_handles WHERE contact_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            let tags = sqlx::query_scalar::<_, String>(
                "SELECT tag_id FROM contact_tags WHERE contact_id = ?",
            )
            .bind(&row.id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let projects = sqlx::query_scalar::<_, String>(
                "SELECT project_id FROM project_contacts WHERE contact_id = ?",
            )
            .bind(&row.id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let groups = sqlx::query_scalar::<_, String>(
                "SELECT group_id FROM contact_groups WHERE contact_id = ?",
            )
            .bind(&row.id)
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let contact = row.into_contact(social_handles, tags, projects, groups);
            duplicates
                .entry(phone)
                .or_insert_with(Vec::new)
                .push(contact);
        }

        // Filter to only include phone numbers with 2+ contacts
        duplicates.retain(|_, contacts| contacts.len() > 1);

        Ok(duplicates)
    }

    /// Merge contact data from merge_id into keep_id, then delete merge_id
    /// Returns statistics about what was merged
    pub async fn merge_contacts(
        &self,
        keep_id: Uuid,
        merge_id: Uuid,
    ) -> DomainResult<MergeStatistics> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut stats = MergeStatistics::default();

        // Get both contacts to merge their data
        let keep_contact = self.get_by_id(keep_id).await?;
        let merge_contact = self.get_by_id(merge_id).await?;

        // 1. Merge notes (append)
        let merged_notes = if let Some(keep_notes) = &keep_contact.notes {
            if let Some(merge_notes) = &merge_contact.notes {
                if !merge_notes.is_empty() {
                    Some(format!(
                        "{}\n\n--- Merged from duplicate contact ---\n{}",
                        keep_notes, merge_notes
                    ))
                } else {
                    Some(keep_notes.clone())
                }
            } else {
                Some(keep_notes.clone())
            }
        } else {
            merge_contact.notes.clone()
        };

        sqlx::query("UPDATE contacts SET notes = ? WHERE id = ?")
            .bind(&merged_notes)
            .bind(keep_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // 2. Merge social_handles (deduplicate by platform)
        let merge_handles = sqlx::query_as::<_, SocialHandleRow>(
            "SELECT platform, handle, url FROM social_handles WHERE contact_id = ?",
        )
        .bind(merge_id.to_string())
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for handle in merge_handles {
            // Only add if keep_contact doesn't already have this platform
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM social_handles WHERE contact_id = ? AND platform = ?",
            )
            .bind(keep_id.to_string())
            .bind(&handle.platform)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            if exists == 0 {
                sqlx::query(
                    "INSERT INTO social_handles (contact_id, platform, handle, url) VALUES (?, ?, ?, ?)"
                )
                .bind(keep_id.to_string())
                .bind(&handle.platform)
                .bind(&handle.handle)
                .bind(&handle.url)
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
                stats.social_handles_merged += 1;
            }
        }

        // 3. Merge tags (union)
        let merge_tags =
            sqlx::query_scalar::<_, String>("SELECT tag_id FROM contact_tags WHERE contact_id = ?")
                .bind(merge_id.to_string())
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;

        for tag_id in merge_tags {
            // Use INSERT OR IGNORE to avoid duplicates
            let result = sqlx::query(
                "INSERT OR IGNORE INTO contact_tags (contact_id, tag_id) VALUES (?, ?)",
            )
            .bind(keep_id.to_string())
            .bind(&tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            if result.rows_affected() > 0 {
                stats.tags_merged += 1;
            }
        }

        // 4. Merge projects (union)
        let merge_projects = sqlx::query_scalar::<_, String>(
            "SELECT project_id FROM project_contacts WHERE contact_id = ?",
        )
        .bind(merge_id.to_string())
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for project_id in merge_projects {
            let result = sqlx::query(
                "INSERT OR IGNORE INTO project_contacts (project_id, contact_id) VALUES (?, ?)",
            )
            .bind(&project_id)
            .bind(keep_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            if result.rows_affected() > 0 {
                stats.projects_merged += 1;
            }
        }

        // 5. Merge groups (union)
        let merge_groups = sqlx::query_scalar::<_, String>(
            "SELECT group_id FROM contact_groups WHERE contact_id = ?",
        )
        .bind(merge_id.to_string())
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for group_id in merge_groups {
            let result = sqlx::query(
                "INSERT OR IGNORE INTO contact_groups (group_id, contact_id) VALUES (?, ?)",
            )
            .bind(&group_id)
            .bind(keep_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            if result.rows_affected() > 0 {
                stats.groups_merged += 1;
            }
        }

        // 6. Re-link sms_history
        let sms_result = sqlx::query("UPDATE sms_history SET contact_id = ? WHERE contact_id = ?")
            .bind(keep_id.to_string())
            .bind(merge_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        stats.sms_messages_relinked = sms_result.rows_affected() as usize;

        // 6b. Deduplicate SMS messages for the kept contact
        // Find and delete duplicate SMS messages based on phone_number, message_date, message_type, and body
        // Keep the one with the lowest id (earliest import)
        let dedupe_result = sqlx::query(
            "DELETE FROM sms_history
             WHERE id IN (
                 SELECT id FROM (
                     SELECT id,
                            ROW_NUMBER() OVER (
                                PARTITION BY phone_number, message_date, message_type, body
                                ORDER BY id
                            ) as rn
                     FROM sms_history
                     WHERE contact_id = ?
                 )
                 WHERE rn > 1
             )",
        )
        .bind(keep_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        stats.sms_duplicates_removed = dedupe_result.rows_affected() as usize;

        // 7. Re-link communication_attempts
        let comm_result =
            sqlx::query("UPDATE communication_attempts SET contact_id = ? WHERE contact_id = ?")
                .bind(keep_id.to_string())
                .bind(merge_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        stats.communications_relinked = comm_result.rows_affected() as usize;

        // 8. Re-link notes table entries
        let notes_result = sqlx::query("UPDATE notes SET contact_id = ? WHERE contact_id = ?")
            .bind(keep_id.to_string())
            .bind(merge_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        stats.notes_relinked = notes_result.rows_affected() as usize;

        // 9. Re-link ai_suggestions
        let ai_result =
            sqlx::query("UPDATE ai_suggestions SET contact_id = ? WHERE contact_id = ?")
                .bind(keep_id.to_string())
                .bind(merge_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        stats.ai_suggestions_relinked = ai_result.rows_affected() as usize;

        // 10. Re-link event_contacts
        let events_result =
            sqlx::query("UPDATE OR IGNORE event_contacts SET contact_id = ? WHERE contact_id = ?")
                .bind(keep_id.to_string())
                .bind(merge_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        stats.events_relinked = events_result.rows_affected() as usize;

        // 11. Delete the duplicate contact (CASCADE will handle social_handles, contact_tags, etc.)
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(merge_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(stats)
    }
}

#[derive(sqlx::FromRow)]
struct ContactRow {
    id: String,
    first_name: String,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
    title: Option<String>,
    notes: Option<String>,
    metadata: String,
    created_at: String,
    updated_at: String,
    created_by: String,
    version: i32,
    last_synced_at: Option<String>,
}

impl ContactRow {
    fn into_contact(
        self,
        social_handles: Vec<SocialHandleRow>,
        tags: Vec<Uuid>,
        projects: Vec<Uuid>,
        groups: Vec<Uuid>,
    ) -> Contact {
        Contact {
            id: Uuid::parse_str(&self.id).unwrap(),
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
            phone: self.phone,
            organization: self.organization,
            title: self.title,
            notes: self.notes,
            social_handles: social_handles.into_iter().map(|h| h.into()).collect(),
            tags,
            projects,
            groups,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
            version: self.version,
            last_synced_at: self.last_synced_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            metadata: serde_json::from_str(&self.metadata).unwrap_or(serde_json::json!({})),
        }
    }
}

#[derive(sqlx::FromRow)]
struct SocialHandleRow {
    platform: String,
    handle: String,
    url: Option<String>,
}

impl From<SocialHandleRow> for SocialHandle {
    fn from(row: SocialHandleRow) -> Self {
        Self {
            platform: row.platform,
            handle: row.handle,
            url: row.url,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MergeStatistics {
    pub social_handles_merged: usize,
    pub tags_merged: usize,
    pub projects_merged: usize,
    pub groups_merged: usize,
    pub sms_messages_relinked: usize,
    pub sms_duplicates_removed: usize,
    pub communications_relinked: usize,
    pub notes_relinked: usize,
    pub ai_suggestions_relinked: usize,
    pub events_relinked: usize,
}
