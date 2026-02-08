use crate::db::DbPool;
use core_domain::{
    ConceptMatcher, ConceptMatcherGroup, DomainError, DomainResult, MatchMode, MatcherElementType,
};
use uuid::Uuid;

/// Helper struct for loading concepts with their matcher groups
#[derive(Debug, Clone)]
pub struct ConceptWithMatchers {
    pub concept_id: Uuid,
    pub concept_name: String,
    pub groups: Vec<ConceptMatcherGroup>,
}

pub struct ConceptMatcherRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> ConceptMatcherRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    // --- Group operations ---

    pub async fn create_group(&self, group: &ConceptMatcherGroup) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO concept_matcher_groups (id, concept_id, position, created_at, created_by)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(group.id.to_string())
        .bind(group.concept_id.to_string())
        .bind(group.position)
        .bind(group.created_at.to_rfc3339())
        .bind(group.created_by.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_groups_by_concept(
        &self,
        concept_id: Uuid,
    ) -> DomainResult<Vec<ConceptMatcherGroup>> {
        let group_rows = sqlx::query_as::<_, MatcherGroupRow>(
            "SELECT id, concept_id, position, created_at, created_by
             FROM concept_matcher_groups
             WHERE concept_id = ?
             ORDER BY position ASC",
        )
        .bind(concept_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut groups = Vec::new();
        for row in group_rows {
            let group_id =
                Uuid::parse_str(&row.id).map_err(|e| DomainError::Internal(e.to_string()))?;
            let matchers = self.list_matchers_by_group(group_id).await?;
            groups.push(ConceptMatcherGroup {
                id: group_id,
                concept_id: Uuid::parse_str(&row.concept_id)
                    .map_err(|e| DomainError::Internal(e.to_string()))?,
                position: row.position,
                matchers,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .map_err(|e| DomainError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                created_by: Uuid::parse_str(&row.created_by)
                    .map_err(|e| DomainError::Internal(e.to_string()))?,
            });
        }

        Ok(groups)
    }

    pub async fn delete_group(&self, group_id: Uuid) -> DomainResult<()> {
        // Matchers are cascade-deleted via FK
        sqlx::query("DELETE FROM concept_matcher_groups WHERE id = ?")
            .bind(group_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    // --- Matcher operations ---

    pub async fn create_matcher(&self, matcher: &ConceptMatcher) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO concept_matchers (id, group_id, element_type, match_value, match_mode, negate, case_sensitive, position, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(matcher.id.to_string())
        .bind(matcher.group_id.to_string())
        .bind(matcher.element_type.to_string())
        .bind(&matcher.match_value)
        .bind(matcher.match_mode.to_string())
        .bind(matcher.negate as i32)
        .bind(matcher.case_sensitive as i32)
        .bind(matcher.position)
        .bind(matcher.created_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn update_matcher(&self, matcher: &ConceptMatcher) -> DomainResult<()> {
        sqlx::query(
            "UPDATE concept_matchers SET element_type = ?, match_value = ?, match_mode = ?, negate = ?, case_sensitive = ?, position = ? WHERE id = ?",
        )
        .bind(matcher.element_type.to_string())
        .bind(&matcher.match_value)
        .bind(matcher.match_mode.to_string())
        .bind(matcher.negate as i32)
        .bind(matcher.case_sensitive as i32)
        .bind(matcher.position)
        .bind(matcher.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_matcher(&self, matcher_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM concept_matchers WHERE id = ?")
            .bind(matcher_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn list_matchers_by_group(
        &self,
        group_id: Uuid,
    ) -> DomainResult<Vec<ConceptMatcher>> {
        let rows = sqlx::query_as::<_, MatcherRow>(
            "SELECT id, group_id, element_type, match_value, match_mode, negate, case_sensitive, position, created_at
             FROM concept_matchers
             WHERE group_id = ?
             ORDER BY position ASC",
        )
        .bind(group_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.into_iter().map(|r| r.into_matcher()).collect()
    }

    // --- Bulk operations ---

    /// Replace all matcher groups+matchers for a concept (full save)
    pub async fn replace_concept_matchers(
        &self,
        concept_id: Uuid,
        groups: Vec<ConceptMatcherGroup>,
    ) -> DomainResult<()> {
        // Delete existing groups (cascades to matchers via FK with pragma)
        // SQLite needs PRAGMA foreign_keys = ON for cascade, so delete matchers manually too
        let existing_groups = self.list_groups_by_concept(concept_id).await?;
        for g in &existing_groups {
            sqlx::query("DELETE FROM concept_matchers WHERE group_id = ?")
                .bind(g.id.to_string())
                .execute(self.pool)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        sqlx::query("DELETE FROM concept_matcher_groups WHERE concept_id = ?")
            .bind(concept_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Insert new groups and matchers
        for group in &groups {
            self.create_group(group).await?;
            for matcher in &group.matchers {
                self.create_matcher(matcher).await?;
            }
        }

        Ok(())
    }

    /// Load all concepts that have at least one matcher group, with their matchers
    pub async fn load_all_concepts_with_matchers(&self) -> DomainResult<Vec<ConceptWithMatchers>> {
        // Find all concept_ids that have matcher groups
        let rows = sqlx::query_as::<_, ConceptIdNameRow>(
            "SELECT DISTINCT c.id, c.name
             FROM concepts c
             INNER JOIN concept_matcher_groups g ON g.concept_id = c.id
             ORDER BY c.name",
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let concept_id =
                Uuid::parse_str(&row.id).map_err(|e| DomainError::Internal(e.to_string()))?;
            let groups = self.list_groups_by_concept(concept_id).await?;
            results.push(ConceptWithMatchers {
                concept_id,
                concept_name: row.name,
                groups,
            });
        }

        Ok(results)
    }
}

// --- Row types ---

#[derive(sqlx::FromRow)]
struct MatcherGroupRow {
    id: String,
    concept_id: String,
    position: i32,
    created_at: String,
    created_by: String,
}

#[derive(sqlx::FromRow)]
struct MatcherRow {
    id: String,
    group_id: String,
    element_type: String,
    match_value: String,
    match_mode: String,
    negate: i32,
    case_sensitive: i32,
    position: i32,
    created_at: String,
}

impl MatcherRow {
    fn into_matcher(self) -> DomainResult<ConceptMatcher> {
        Ok(ConceptMatcher {
            id: Uuid::parse_str(&self.id).map_err(|e| DomainError::Internal(e.to_string()))?,
            group_id: Uuid::parse_str(&self.group_id)
                .map_err(|e| DomainError::Internal(e.to_string()))?,
            element_type: self
                .element_type
                .parse::<MatcherElementType>()
                .map_err(DomainError::Internal)?,
            match_value: self.match_value,
            match_mode: self
                .match_mode
                .parse::<MatchMode>()
                .map_err(DomainError::Internal)?,
            negate: self.negate != 0,
            case_sensitive: self.case_sensitive != 0,
            position: self.position,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| DomainError::Internal(e.to_string()))?
                .with_timezone(&chrono::Utc),
        })
    }
}

#[derive(sqlx::FromRow)]
struct ConceptIdNameRow {
    id: String,
    name: String,
}
