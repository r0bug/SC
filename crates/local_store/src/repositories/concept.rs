use core_domain::{Concept, DomainError, DomainResult};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct ConceptRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> ConceptRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, concept: &Concept) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO concepts (id, name, description, tags, created_at, updated_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(concept.id.to_string())
        .bind(&concept.name)
        .bind(&concept.description)
        .bind(serde_json::to_string(&concept.tags).unwrap())
        .bind(concept.created_at.to_rfc3339())
        .bind(concept.updated_at.to_rfc3339())
        .bind(concept.created_by.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for contact_id in &concept.related_contacts {
            sqlx::query("INSERT INTO concept_contacts (concept_id, contact_id) VALUES (?, ?)")
                .bind(concept.id.to_string())
                .bind(contact_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        for project_id in &concept.related_projects {
            sqlx::query("INSERT INTO concept_projects (concept_id, project_id) VALUES (?, ?)")
                .bind(concept.id.to_string())
                .bind(project_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Concept> {
        let row = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, tags, created_at, updated_at, created_by
             FROM concepts WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Concept {}", id)))?;

        let related_contacts = sqlx::query_scalar::<_, String>(
            "SELECT contact_id FROM concept_contacts WHERE concept_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        let related_projects = sqlx::query_scalar::<_, String>(
            "SELECT project_id FROM concept_projects WHERE concept_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        Ok(row.into_concept(related_contacts, related_projects))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, tags, created_at, updated_at, created_by
             FROM concepts ORDER BY name LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut concepts = Vec::new();
        for row in rows {
            let _concept_id = Uuid::parse_str(&row.id).unwrap();

            let related_contacts = sqlx::query_scalar::<_, String>(
                "SELECT contact_id FROM concept_contacts WHERE concept_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let related_projects = sqlx::query_scalar::<_, String>(
                "SELECT project_id FROM concept_projects WHERE concept_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            concepts.push(row.into_concept(related_contacts, related_projects));
        }

        Ok(concepts)
    }

    pub async fn update(&self, concept: &Concept) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE concepts SET name = ?, description = ?, tags = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&concept.name)
        .bind(&concept.description)
        .bind(serde_json::to_string(&concept.tags).unwrap())
        .bind(concept.updated_at.to_rfc3339())
        .bind(concept.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM concept_contacts WHERE concept_id = ?")
            .bind(concept.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for contact_id in &concept.related_contacts {
            sqlx::query("INSERT INTO concept_contacts (concept_id, contact_id) VALUES (?, ?)")
                .bind(concept.id.to_string())
                .bind(contact_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        sqlx::query("DELETE FROM concept_projects WHERE concept_id = ?")
            .bind(concept.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for project_id in &concept.related_projects {
            sqlx::query("INSERT INTO concept_projects (concept_id, project_id) VALUES (?, ?)")
                .bind(concept.id.to_string())
                .bind(project_id.to_string())
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
        sqlx::query("DELETE FROM concepts WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_contact(&self, concept_id: Uuid, contact_id: Uuid) -> DomainResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO concept_contacts (concept_id, contact_id) VALUES (?, ?)",
        )
        .bind(concept_id.to_string())
        .bind(contact_id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_contact(&self, concept_id: Uuid, contact_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM concept_contacts WHERE concept_id = ? AND contact_id = ?")
            .bind(concept_id.to_string())
            .bind(contact_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_project(&self, concept_id: Uuid, project_id: Uuid) -> DomainResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO concept_projects (concept_id, project_id) VALUES (?, ?)",
        )
        .bind(concept_id.to_string())
        .bind(project_id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_project(&self, concept_id: Uuid, project_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM concept_projects WHERE concept_id = ? AND project_id = ?")
            .bind(concept_id.to_string())
            .bind(project_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_creator(&self, creator_id: Uuid) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, tags, created_at, updated_at, created_by
             FROM concepts WHERE created_by = ? ORDER BY name",
        )
        .bind(creator_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut concepts = Vec::new();
        for row in rows {
            let concept = self.build_concept_from_row(row).await?;
            concepts.push(concept);
        }

        Ok(concepts)
    }

    pub async fn search(&self, query: &str) -> DomainResult<Vec<Concept>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, tags, created_at, updated_at, created_by
             FROM concepts
             WHERE name LIKE ? OR description LIKE ? OR tags LIKE ?
             ORDER BY name
             LIMIT 50",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut concepts = Vec::new();
        for row in rows {
            let concept = self.build_concept_from_row(row).await?;
            concepts.push(concept);
        }

        Ok(concepts)
    }

    async fn build_concept_from_row(&self, row: ConceptRow) -> DomainResult<Concept> {
        let related_contacts = sqlx::query_scalar::<_, String>(
            "SELECT contact_id FROM concept_contacts WHERE concept_id = ?",
        )
        .bind(&row.id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        let related_projects = sqlx::query_scalar::<_, String>(
            "SELECT project_id FROM concept_projects WHERE concept_id = ?",
        )
        .bind(&row.id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        Ok(row.into_concept(related_contacts, related_projects))
    }
}

#[derive(sqlx::FromRow)]
struct ConceptRow {
    id: String,
    name: String,
    description: Option<String>,
    tags: String,
    created_at: String,
    updated_at: String,
    created_by: String,
}

impl ConceptRow {
    fn into_concept(self, related_contacts: Vec<Uuid>, related_projects: Vec<Uuid>) -> Concept {
        Concept {
            id: Uuid::parse_str(&self.id).unwrap(),
            name: self.name,
            description: self.description,
            related_contacts,
            related_projects,
            tags: serde_json::from_str(&self.tags).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(crate::migrations::SCHEMA)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_and_get_concept() {
        let pool = setup_test_db().await;
        let repo = ConceptRepository::new(&pool);

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let concept = Concept {
            id: Uuid::new_v4(),
            name: "Test Concept".to_string(),
            description: Some("A test concept".to_string()),
            related_contacts: vec![],
            related_projects: vec![],
            tags: vec!["test".to_string(), "demo".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&concept).await.unwrap();

        let retrieved = repo.get_by_id(concept.id).await.unwrap();
        assert_eq!(retrieved.name, concept.name);
        assert_eq!(retrieved.tags.len(), 2);
    }

    #[tokio::test]
    async fn test_add_and_remove_relationships() {
        let pool = setup_test_db().await;
        let repo = ConceptRepository::new(&pool);

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        let contact_id = Uuid::new_v4();
        sqlx::query("INSERT INTO contacts (id, first_name, created_at, updated_at, created_by, version) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(contact_id.to_string())
            .bind("John")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let project_id = Uuid::new_v4();
        sqlx::query("INSERT INTO projects (id, name, status, created_at, updated_at, created_by, version) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(project_id.to_string())
            .bind("Test Project")
            .bind("Active")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(user_id.to_string())
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

        let concept = Concept {
            id: Uuid::new_v4(),
            name: "Test Concept".to_string(),
            description: None,
            related_contacts: vec![],
            related_projects: vec![],
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&concept).await.unwrap();
        repo.add_contact(concept.id, contact_id).await.unwrap();
        repo.add_project(concept.id, project_id).await.unwrap();

        let retrieved = repo.get_by_id(concept.id).await.unwrap();
        assert_eq!(retrieved.related_contacts.len(), 1);
        assert_eq!(retrieved.related_projects.len(), 1);

        repo.remove_contact(concept.id, contact_id).await.unwrap();
        repo.remove_project(concept.id, project_id).await.unwrap();

        let retrieved = repo.get_by_id(concept.id).await.unwrap();
        assert_eq!(retrieved.related_contacts.len(), 0);
        assert_eq!(retrieved.related_projects.len(), 0);
    }
}
