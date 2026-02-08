use crate::db::DbPool;
use core_domain::{Concept, ConceptKeyword, DomainError, DomainResult};
use uuid::Uuid;

pub struct ConceptRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> ConceptRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, concept: &Concept) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO concepts (id, name, description, parent_id, metadata, created_at, updated_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(concept.id.to_string())
        .bind(&concept.name)
        .bind(&concept.description)
        .bind(concept.parent_id.map(|id| id.to_string()))
        .bind(serde_json::to_string(&concept.metadata).unwrap_or_else(|_| "{}".to_string()))
        .bind(concept.created_at.to_rfc3339())
        .bind(concept.updated_at.to_rfc3339())
        .bind(concept.created_by.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for kw in &concept.keywords {
            sqlx::query(
                "INSERT INTO concept_keywords (id, concept_id, keyword, created_at) VALUES (?, ?, ?, ?)",
            )
            .bind(kw.id.to_string())
            .bind(concept.id.to_string())
            .bind(&kw.keyword)
            .bind(kw.created_at.to_rfc3339())
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
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Concept {}", id)))?;

        let keywords = self.get_keywords_for_concept(id).await?;
        Ok(row.into_concept(keywords))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts ORDER BY name LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        // Batch-load keywords for all concepts
        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    pub async fn list_by_creator(&self, creator_id: Uuid) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts WHERE created_by = ? ORDER BY name",
        )
        .bind(creator_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    pub async fn list_children(&self, parent_id: Uuid) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts WHERE parent_id = ? ORDER BY name",
        )
        .bind(parent_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    pub async fn list_roots(&self) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts WHERE parent_id IS NULL ORDER BY name",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    pub async fn update(&self, concept: &Concept) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE concepts SET name = ?, description = ?, parent_id = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&concept.name)
        .bind(&concept.description)
        .bind(concept.parent_id.map(|id| id.to_string()))
        .bind(serde_json::to_string(&concept.metadata).unwrap_or_else(|_| "{}".to_string()))
        .bind(concept.updated_at.to_rfc3339())
        .bind(concept.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Replace keywords: delete all, re-insert
        sqlx::query("DELETE FROM concept_keywords WHERE concept_id = ?")
            .bind(concept.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for kw in &concept.keywords {
            sqlx::query(
                "INSERT INTO concept_keywords (id, concept_id, keyword, created_at) VALUES (?, ?, ?, ?)",
            )
            .bind(kw.id.to_string())
            .bind(concept.id.to_string())
            .bind(&kw.keyword)
            .bind(kw.created_at.to_rfc3339())
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
        // Keywords cascade via FK, but be explicit
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM concept_keywords WHERE concept_id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM concepts WHERE id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_keyword(
        &self,
        concept_id: Uuid,
        keyword: &ConceptKeyword,
    ) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO concept_keywords (id, concept_id, keyword, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(keyword.id.to_string())
        .bind(concept_id.to_string())
        .bind(&keyword.keyword)
        .bind(keyword.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_keyword(&self, keyword_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM concept_keywords WHERE id = ?")
            .bind(keyword_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn search(&self, query: &str) -> DomainResult<Vec<Concept>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT DISTINCT c.id, c.name, c.description, c.parent_id, c.metadata, c.created_at, c.updated_at, c.created_by
             FROM concepts c
             LEFT JOIN concept_keywords ck ON c.id = ck.concept_id
             WHERE c.name LIKE ? OR c.description LIKE ? OR ck.keyword LIKE ?
             ORDER BY c.name
             LIMIT 50",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    /// Find concepts matching a keyword exactly (for detection engine)
    pub async fn find_by_keyword(&self, keyword: &str) -> DomainResult<Vec<Concept>> {
        let lower = keyword.to_lowercase();
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT DISTINCT c.id, c.name, c.description, c.parent_id, c.metadata, c.created_at, c.updated_at, c.created_by
             FROM concepts c
             INNER JOIN concept_keywords ck ON c.id = ck.concept_id
             WHERE LOWER(ck.keyword) = ?
             ORDER BY c.name",
        )
        .bind(&lower)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    /// Search communication domains by name or keyword (for autocomplete)
    pub async fn search_communication_domains(&self, query: &str) -> DomainResult<Vec<Concept>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT DISTINCT c.id, c.name, c.description, c.parent_id, c.metadata, c.created_at, c.updated_at, c.created_by
             FROM concepts c
             LEFT JOIN concept_keywords ck ON c.id = ck.concept_id
             WHERE json_extract(c.metadata, '$.concept_type') = 'communication_domain'
               AND (c.name LIKE ? OR ck.keyword LIKE ?)
             ORDER BY c.name
             LIMIT 20",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    /// Find a communication domain by exact name (case-insensitive) for deduplication
    pub async fn find_communication_domain_by_name(
        &self,
        name: &str,
    ) -> DomainResult<Option<Concept>> {
        let lower = name.to_lowercase();
        let row = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts
             WHERE json_extract(metadata, '$.concept_type') = 'communication_domain'
               AND LOWER(name) = ?
             LIMIT 1",
        )
        .bind(&lower)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        match row {
            Some(r) => {
                let id_str = r.id.clone();
                let keywords = self.batch_get_keywords(&[id_str]).await?;
                let kws = keywords.get(&r.id).cloned().unwrap_or_default();
                Ok(Some(r.into_concept(kws)))
            }
            None => Ok(None),
        }
    }

    /// List concepts that are communication domains (metadata contains concept_type = "communication_domain")
    pub async fn list_communication_domains(&self) -> DomainResult<Vec<Concept>> {
        let rows = sqlx::query_as::<_, ConceptRow>(
            "SELECT id, name, description, parent_id, metadata, created_at, updated_at, created_by
             FROM concepts
             WHERE json_extract(metadata, '$.concept_type') = 'communication_domain'
             ORDER BY name",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let concept_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let keywords_map = self.batch_get_keywords(&concept_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let kws = keywords_map.get(&row.id).cloned().unwrap_or_default();
                row.into_concept(kws)
            })
            .collect())
    }

    // --- Internal helpers ---

    async fn get_keywords_for_concept(
        &self,
        concept_id: Uuid,
    ) -> DomainResult<Vec<ConceptKeyword>> {
        let rows = sqlx::query_as::<_, KeywordRow>(
            "SELECT id, concept_id, keyword, created_at FROM concept_keywords WHERE concept_id = ?",
        )
        .bind(concept_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_keyword()).collect())
    }

    async fn batch_get_keywords(
        &self,
        concept_ids: &[String],
    ) -> DomainResult<std::collections::HashMap<String, Vec<ConceptKeyword>>> {
        if concept_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let placeholders: Vec<&str> = concept_ids.iter().map(|_| "?").collect();
        let query = format!(
            "SELECT id, concept_id, keyword, created_at FROM concept_keywords WHERE concept_id IN ({})",
            placeholders.join(", ")
        );

        let mut q = sqlx::query_as::<_, KeywordRow>(&query);
        for id in concept_ids {
            q = q.bind(id);
        }

        let rows = q
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut map: std::collections::HashMap<String, Vec<ConceptKeyword>> =
            std::collections::HashMap::new();
        for row in rows {
            let cid = row.concept_id.clone();
            map.entry(cid).or_default().push(row.into_keyword());
        }

        Ok(map)
    }
}

#[derive(sqlx::FromRow)]
struct ConceptRow {
    id: String,
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
    metadata: String,
    created_at: String,
    updated_at: String,
    created_by: String,
}

impl ConceptRow {
    fn into_concept(self, keywords: Vec<ConceptKeyword>) -> Concept {
        Concept {
            id: Uuid::parse_str(&self.id).unwrap(),
            name: self.name,
            description: self.description,
            parent_id: self.parent_id.and_then(|s| Uuid::parse_str(&s).ok()),
            keywords,
            metadata: serde_json::from_str(&self.metadata).unwrap_or_default(),
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

#[derive(sqlx::FromRow)]
struct KeywordRow {
    id: String,
    concept_id: String,
    keyword: String,
    created_at: String,
}

impl KeywordRow {
    fn into_keyword(self) -> ConceptKeyword {
        ConceptKeyword {
            id: Uuid::parse_str(&self.id).unwrap(),
            concept_id: Uuid::parse_str(&self.concept_id).unwrap(),
            keyword: self.keyword,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> DbPool {
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

    async fn create_test_user(pool: &DbPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id.to_string())
            .bind("test@example.com")
            .bind("Test")
            .bind("hash")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await
            .unwrap();
        user_id
    }

    #[tokio::test]
    async fn test_create_and_get_concept() {
        let pool = setup_test_db().await;
        let repo = ConceptRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let concept = Concept {
            id: Uuid::new_v4(),
            name: "Military".to_string(),
            description: Some("Military collectibles".to_string()),
            parent_id: None,
            keywords: vec![
                ConceptKeyword {
                    id: Uuid::new_v4(),
                    concept_id: Uuid::nil(), // will be set by concept.id
                    keyword: "Trench Art".to_string(),
                    created_at: Utc::now(),
                },
                ConceptKeyword {
                    id: Uuid::new_v4(),
                    concept_id: Uuid::nil(),
                    keyword: "Medal".to_string(),
                    created_at: Utc::now(),
                },
            ],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&concept).await.unwrap();

        let retrieved = repo.get_by_id(concept.id).await.unwrap();
        assert_eq!(retrieved.name, "Military");
        assert_eq!(retrieved.keywords.len(), 2);
        assert!(retrieved.parent_id.is_none());
    }

    #[tokio::test]
    async fn test_hierarchy() {
        let pool = setup_test_db().await;
        let repo = ConceptRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let parent = Concept {
            id: Uuid::new_v4(),
            name: "Military".to_string(),
            description: None,
            parent_id: None,
            keywords: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&parent).await.unwrap();

        let child = Concept {
            id: Uuid::new_v4(),
            name: "WWII".to_string(),
            description: None,
            parent_id: Some(parent.id),
            keywords: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&child).await.unwrap();

        let children = repo.list_children(parent.id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "WWII");

        let roots = repo.list_roots().await.unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "Military");
    }

    #[tokio::test]
    async fn test_keyword_management() {
        let pool = setup_test_db().await;
        let repo = ConceptRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let concept = Concept {
            id: Uuid::new_v4(),
            name: "Marbles".to_string(),
            description: None,
            parent_id: None,
            keywords: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&concept).await.unwrap();

        let kw = ConceptKeyword {
            id: Uuid::new_v4(),
            concept_id: concept.id,
            keyword: "Shooter".to_string(),
            created_at: Utc::now(),
        };
        repo.add_keyword(concept.id, &kw).await.unwrap();

        let retrieved = repo.get_by_id(concept.id).await.unwrap();
        assert_eq!(retrieved.keywords.len(), 1);
        assert_eq!(retrieved.keywords[0].keyword, "Shooter");

        repo.remove_keyword(kw.id).await.unwrap();
        let retrieved = repo.get_by_id(concept.id).await.unwrap();
        assert_eq!(retrieved.keywords.len(), 0);
    }

    #[tokio::test]
    async fn test_search_with_keywords() {
        let pool = setup_test_db().await;
        let repo = ConceptRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let concept = Concept {
            id: Uuid::new_v4(),
            name: "Military".to_string(),
            description: None,
            parent_id: None,
            keywords: vec![ConceptKeyword {
                id: Uuid::new_v4(),
                concept_id: Uuid::nil(),
                keyword: "Trench Art".to_string(),
                created_at: Utc::now(),
            }],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&concept).await.unwrap();

        // Search by name
        let results = repo.search("Milit").await.unwrap();
        assert_eq!(results.len(), 1);

        // Search by keyword
        let results = repo.search("Trench").await.unwrap();
        assert_eq!(results.len(), 1);

        // Find by exact keyword
        let results = repo.find_by_keyword("trench art").await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
