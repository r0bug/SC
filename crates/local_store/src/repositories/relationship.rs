use core_domain::{DomainError, DomainResult, EntityRelationship, EntityType, RelationshipType};
use crate::db::DbPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// RelationshipTypeRepository
// ---------------------------------------------------------------------------

pub struct RelationshipTypeRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> RelationshipTypeRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> DomainResult<Vec<RelationshipType>> {
        let rows = sqlx::query_as::<_, RelationshipTypeRow>(
            "SELECT id, name, description, is_core, created_at
             FROM relationship_types ORDER BY name",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<RelationshipType> {
        let row = sqlx::query_as::<_, RelationshipTypeRow>(
            "SELECT id, name, description, is_core, created_at
             FROM relationship_types WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("RelationshipType {}", id)))?;

        Ok(row.into())
    }

    pub async fn get_by_name(&self, name: &str) -> DomainResult<RelationshipType> {
        let row = sqlx::query_as::<_, RelationshipTypeRow>(
            "SELECT id, name, description, is_core, created_at
             FROM relationship_types WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("RelationshipType with name '{}'", name)))?;

        Ok(row.into())
    }

    pub async fn create(&self, rel_type: &RelationshipType) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO relationship_types (id, name, description, is_core, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(rel_type.id.to_string())
        .bind(&rel_type.name)
        .bind(&rel_type.description)
        .bind(rel_type.is_core as i32)
        .bind(rel_type.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        // Only allow deletion of non-core relationship types
        let rel_type = self.get_by_id(id).await?;
        if rel_type.is_core {
            return Err(DomainError::Validation(
                "Cannot delete a core relationship type".to_string(),
            ));
        }

        sqlx::query("DELETE FROM relationship_types WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RelationshipTypeRow {
    id: String,
    name: String,
    description: Option<String>,
    is_core: i32,
    created_at: String,
}

impl From<RelationshipTypeRow> for RelationshipType {
    fn from(row: RelationshipTypeRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            name: row.name,
            description: row.description,
            is_core: row.is_core != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}

// ---------------------------------------------------------------------------
// EntityRelationshipRepository
// ---------------------------------------------------------------------------

pub struct EntityRelationshipRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> EntityRelationshipRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, rel: &EntityRelationship) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO entity_relationships
                (id, source_type, source_id, relationship_type_id, target_type, target_id, metadata, created_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(rel.id.to_string())
        .bind(rel.source_type.to_string())
        .bind(rel.source_id.to_string())
        .bind(rel.relationship_type_id.to_string())
        .bind(rel.target_type.to_string())
        .bind(rel.target_id.to_string())
        .bind(serde_json::to_string(&rel.metadata).unwrap_or_else(|_| "{}".to_string()))
        .bind(rel.created_at.to_rfc3339())
        .bind(rel.created_by.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<EntityRelationship> {
        let row = sqlx::query_as::<_, EntityRelationshipRow>(
            "SELECT er.id, er.source_type, er.source_id, er.relationship_type_id,
                    rt.name AS relationship_type_name,
                    er.target_type, er.target_id, er.metadata, er.created_at, er.created_by
             FROM entity_relationships er
             JOIN relationship_types rt ON rt.id = er.relationship_type_id
             WHERE er.id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("EntityRelationship {}", id)))?;

        row.into_entity_relationship()
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM entity_relationships WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_source(
        &self,
        source_type: &EntityType,
        source_id: Uuid,
    ) -> DomainResult<Vec<EntityRelationship>> {
        let rows = sqlx::query_as::<_, EntityRelationshipRow>(
            "SELECT er.id, er.source_type, er.source_id, er.relationship_type_id,
                    rt.name AS relationship_type_name,
                    er.target_type, er.target_id, er.metadata, er.created_at, er.created_by
             FROM entity_relationships er
             JOIN relationship_types rt ON rt.id = er.relationship_type_id
             WHERE er.source_type = ? AND er.source_id = ?
             ORDER BY er.created_at DESC",
        )
        .bind(source_type.to_string())
        .bind(source_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|r| r.into_entity_relationship()).collect()
    }

    pub async fn list_by_target(
        &self,
        target_type: &EntityType,
        target_id: Uuid,
    ) -> DomainResult<Vec<EntityRelationship>> {
        let rows = sqlx::query_as::<_, EntityRelationshipRow>(
            "SELECT er.id, er.source_type, er.source_id, er.relationship_type_id,
                    rt.name AS relationship_type_name,
                    er.target_type, er.target_id, er.metadata, er.created_at, er.created_by
             FROM entity_relationships er
             JOIN relationship_types rt ON rt.id = er.relationship_type_id
             WHERE er.target_type = ? AND er.target_id = ?
             ORDER BY er.created_at DESC",
        )
        .bind(target_type.to_string())
        .bind(target_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|r| r.into_entity_relationship()).collect()
    }

    pub async fn list_by_type(
        &self,
        relationship_type_id: Uuid,
    ) -> DomainResult<Vec<EntityRelationship>> {
        let rows = sqlx::query_as::<_, EntityRelationshipRow>(
            "SELECT er.id, er.source_type, er.source_id, er.relationship_type_id,
                    rt.name AS relationship_type_name,
                    er.target_type, er.target_id, er.metadata, er.created_at, er.created_by
             FROM entity_relationships er
             JOIN relationship_types rt ON rt.id = er.relationship_type_id
             WHERE er.relationship_type_id = ?
             ORDER BY er.created_at DESC",
        )
        .bind(relationship_type_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|r| r.into_entity_relationship()).collect()
    }

    /// Find entities that HAVE what the source WANTs.
    ///
    /// Given a source entity that has a WANT relationship to some target concept,
    /// this finds other entity_relationships where *other* entities HAVE that same
    /// target concept. `want_type_id` is the relationship type for "WANT" and
    /// `have_type_id` is the relationship type for "HAVE".
    pub async fn find_matches(
        &self,
        source_type: &EntityType,
        source_id: Uuid,
        want_type_id: Uuid,
        have_type_id: Uuid,
    ) -> DomainResult<Vec<EntityRelationship>> {
        let rows = sqlx::query_as::<_, EntityRelationshipRow>(
            "SELECT h.id, h.source_type, h.source_id, h.relationship_type_id,
                    rt.name AS relationship_type_name,
                    h.target_type, h.target_id, h.metadata, h.created_at, h.created_by
             FROM entity_relationships w
             JOIN entity_relationships h
               ON h.target_type = w.target_type
              AND h.target_id = w.target_id
              AND h.relationship_type_id = ?
             JOIN relationship_types rt ON rt.id = h.relationship_type_id
             WHERE w.source_type = ?
               AND w.source_id = ?
               AND w.relationship_type_id = ?
             ORDER BY h.created_at DESC",
        )
        .bind(have_type_id.to_string())
        .bind(source_type.to_string())
        .bind(source_id.to_string())
        .bind(want_type_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|r| r.into_entity_relationship()).collect()
    }
}

#[derive(sqlx::FromRow)]
struct EntityRelationshipRow {
    id: String,
    source_type: String,
    source_id: String,
    relationship_type_id: String,
    relationship_type_name: Option<String>,
    target_type: String,
    target_id: String,
    metadata: String,
    created_at: String,
    created_by: String,
}

impl EntityRelationshipRow {
    fn into_entity_relationship(self) -> DomainResult<EntityRelationship> {
        let source_type = self
            .source_type
            .parse::<EntityType>()
            .map_err(|e| DomainError::Internal(e))?;
        let target_type = self
            .target_type
            .parse::<EntityType>()
            .map_err(|e| DomainError::Internal(e))?;

        Ok(EntityRelationship {
            id: Uuid::parse_str(&self.id).unwrap(),
            source_type,
            source_id: Uuid::parse_str(&self.source_id).unwrap(),
            relationship_type_id: Uuid::parse_str(&self.relationship_type_id).unwrap(),
            relationship_type_name: self.relationship_type_name,
            target_type,
            target_id: Uuid::parse_str(&self.target_id).unwrap(),
            metadata: serde_json::from_str(&self.metadata).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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

    async fn insert_test_user(pool: &DbPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id.to_string())
        .bind("test@example.com")
        .bind("Test User")
        .bind("hash")
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap();
        user_id
    }

    // ---- RelationshipTypeRepository tests ----

    #[tokio::test]
    async fn test_create_and_get_relationship_type() {
        let pool = setup_test_db().await;
        let repo = RelationshipTypeRepository::new(&pool);

        let rt = RelationshipType {
            id: Uuid::new_v4(),
            name: "CUSTOM_REL".to_string(),
            description: Some("A custom relationship".to_string()),
            is_core: false,
            created_at: Utc::now(),
        };

        repo.create(&rt).await.unwrap();

        let retrieved = repo.get_by_id(rt.id).await.unwrap();
        assert_eq!(retrieved.name, "CUSTOM_REL");
        assert_eq!(retrieved.description, Some("A custom relationship".to_string()));
        assert!(!retrieved.is_core);
    }

    #[tokio::test]
    async fn test_get_relationship_type_by_name() {
        let pool = setup_test_db().await;
        let repo = RelationshipTypeRepository::new(&pool);

        // Core types are seeded by the schema
        let want = repo.get_by_name("WANT").await.unwrap();
        assert_eq!(want.name, "WANT");
        assert!(want.is_core);
    }

    #[tokio::test]
    async fn test_list_relationship_types() {
        let pool = setup_test_db().await;
        let repo = RelationshipTypeRepository::new(&pool);

        let types = repo.list().await.unwrap();
        // Schema seeds multiple core types
        assert!(types.len() >= 2);
    }

    #[tokio::test]
    async fn test_delete_non_core_relationship_type() {
        let pool = setup_test_db().await;
        let repo = RelationshipTypeRepository::new(&pool);

        let rt = RelationshipType {
            id: Uuid::new_v4(),
            name: "DELETEABLE".to_string(),
            description: None,
            is_core: false,
            created_at: Utc::now(),
        };

        repo.create(&rt).await.unwrap();
        repo.delete(rt.id).await.unwrap();

        let result = repo.get_by_id(rt.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cannot_delete_core_relationship_type() {
        let pool = setup_test_db().await;
        let repo = RelationshipTypeRepository::new(&pool);

        let want = repo.get_by_name("WANT").await.unwrap();
        let result = repo.delete(want.id).await;
        assert!(result.is_err());
    }

    // ---- EntityRelationshipRepository tests ----

    #[tokio::test]
    async fn test_create_and_get_entity_relationship() {
        let pool = setup_test_db().await;
        let user_id = insert_test_user(&pool).await;
        let type_repo = RelationshipTypeRepository::new(&pool);
        let repo = EntityRelationshipRepository::new(&pool);

        let want = type_repo.get_by_name("WANT").await.unwrap();

        let rel = EntityRelationship {
            id: Uuid::new_v4(),
            source_type: EntityType::Contact,
            source_id: Uuid::new_v4(),
            relationship_type_id: want.id,
            relationship_type_name: None,
            target_type: EntityType::Concept,
            target_id: Uuid::new_v4(),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&rel).await.unwrap();

        let retrieved = repo.get_by_id(rel.id).await.unwrap();
        assert_eq!(retrieved.source_type, EntityType::Contact);
        assert_eq!(retrieved.target_type, EntityType::Concept);
        assert_eq!(retrieved.relationship_type_name, Some("WANT".to_string()));
    }

    #[tokio::test]
    async fn test_delete_entity_relationship() {
        let pool = setup_test_db().await;
        let user_id = insert_test_user(&pool).await;
        let type_repo = RelationshipTypeRepository::new(&pool);
        let repo = EntityRelationshipRepository::new(&pool);

        let want = type_repo.get_by_name("WANT").await.unwrap();

        let rel = EntityRelationship {
            id: Uuid::new_v4(),
            source_type: EntityType::Contact,
            source_id: Uuid::new_v4(),
            relationship_type_id: want.id,
            relationship_type_name: None,
            target_type: EntityType::Concept,
            target_id: Uuid::new_v4(),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&rel).await.unwrap();
        repo.delete(rel.id).await.unwrap();

        let result = repo.get_by_id(rel.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_by_source() {
        let pool = setup_test_db().await;
        let user_id = insert_test_user(&pool).await;
        let type_repo = RelationshipTypeRepository::new(&pool);
        let repo = EntityRelationshipRepository::new(&pool);

        let want = type_repo.get_by_name("WANT").await.unwrap();
        let source_id = Uuid::new_v4();

        for _ in 0..3 {
            let rel = EntityRelationship {
                id: Uuid::new_v4(),
                source_type: EntityType::Contact,
                source_id,
                relationship_type_id: want.id,
                relationship_type_name: None,
                target_type: EntityType::Concept,
                target_id: Uuid::new_v4(),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                created_by: user_id,
            };
            repo.create(&rel).await.unwrap();
        }

        let results = repo.list_by_source(&EntityType::Contact, source_id).await.unwrap();
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.relationship_type_name, Some("WANT".to_string()));
        }
    }

    #[tokio::test]
    async fn test_list_by_target() {
        let pool = setup_test_db().await;
        let user_id = insert_test_user(&pool).await;
        let type_repo = RelationshipTypeRepository::new(&pool);
        let repo = EntityRelationshipRepository::new(&pool);

        let have = type_repo.get_by_name("HAVE").await.unwrap();
        let target_id = Uuid::new_v4();

        for _ in 0..2 {
            let rel = EntityRelationship {
                id: Uuid::new_v4(),
                source_type: EntityType::Contact,
                source_id: Uuid::new_v4(),
                relationship_type_id: have.id,
                relationship_type_name: None,
                target_type: EntityType::Concept,
                target_id,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                created_by: user_id,
            };
            repo.create(&rel).await.unwrap();
        }

        let results = repo.list_by_target(&EntityType::Concept, target_id).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let pool = setup_test_db().await;
        let user_id = insert_test_user(&pool).await;
        let type_repo = RelationshipTypeRepository::new(&pool);
        let repo = EntityRelationshipRepository::new(&pool);

        let is_type = type_repo.get_by_name("IS").await.unwrap();

        let rel = EntityRelationship {
            id: Uuid::new_v4(),
            source_type: EntityType::Contact,
            source_id: Uuid::new_v4(),
            relationship_type_id: is_type.id,
            relationship_type_name: None,
            target_type: EntityType::Concept,
            target_id: Uuid::new_v4(),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&rel).await.unwrap();

        let results = repo.list_by_type(is_type.id).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].relationship_type_name, Some("IS".to_string()));
    }

    #[tokio::test]
    async fn test_find_matches() {
        let pool = setup_test_db().await;
        let user_id = insert_test_user(&pool).await;
        let type_repo = RelationshipTypeRepository::new(&pool);
        let repo = EntityRelationshipRepository::new(&pool);

        let want = type_repo.get_by_name("WANT").await.unwrap();
        let have = type_repo.get_by_name("HAVE").await.unwrap();

        let concept_id = Uuid::new_v4();
        let alice_id = Uuid::new_v4();
        let bob_id = Uuid::new_v4();

        // Alice WANTs the concept
        let alice_want = EntityRelationship {
            id: Uuid::new_v4(),
            source_type: EntityType::Contact,
            source_id: alice_id,
            relationship_type_id: want.id,
            relationship_type_name: None,
            target_type: EntityType::Concept,
            target_id: concept_id,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&alice_want).await.unwrap();

        // Bob HAS the concept
        let bob_have = EntityRelationship {
            id: Uuid::new_v4(),
            source_type: EntityType::Contact,
            source_id: bob_id,
            relationship_type_id: have.id,
            relationship_type_name: None,
            target_type: EntityType::Concept,
            target_id: concept_id,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            created_by: user_id,
        };
        repo.create(&bob_have).await.unwrap();

        // Find entities that HAVE what Alice WANTs
        let matches = repo
            .find_matches(&EntityType::Contact, alice_id, want.id, have.id)
            .await
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].source_id, bob_id);
        assert_eq!(matches[0].relationship_type_name, Some("HAVE".to_string()));
    }
}
