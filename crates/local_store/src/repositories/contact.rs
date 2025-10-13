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

            Ok(Some(row.into_contact(social_handles, tags, projects, groups)))
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

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<Contact>> {
        let rows = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts ORDER BY last_name, first_name LIMIT ? OFFSET ?"
        )
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

    pub async fn search(&self, query: &str) -> DomainResult<Vec<Contact>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, ContactRow>(
            "SELECT id, first_name, last_name, email, phone, organization, title, notes, metadata, created_at, updated_at, created_by, version, last_synced_at
             FROM contacts
             WHERE first_name LIKE ? OR last_name LIKE ? OR email LIKE ? OR phone LIKE ? OR organization LIKE ?
             ORDER BY last_name, first_name"
        )
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
