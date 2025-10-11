use core_domain::{DomainError, DomainResult, Project};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct ProjectRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> ProjectRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, project: &Project) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(project.id.to_string())
        .bind(&project.name)
        .bind(&project.description)
        .bind(project.created_at.to_rfc3339())
        .bind(project.updated_at.to_rfc3339())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn list(&self) -> DomainResult<Vec<Project>> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            "SELECT id, name, description, status, created_at, updated_at, created_by, version, last_synced_at FROM projects ORDER BY name"
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut projects = Vec::new();
        for row in rows {
            let contacts = sqlx::query_scalar::<_, String>(
                "SELECT contact_id FROM project_contacts WHERE project_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let tags = sqlx::query_scalar::<_, String>(
                "SELECT tag_id FROM project_tags WHERE project_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            projects.push(row.into_project(contacts, tags, vec![]));
        }

        Ok(projects)
    }
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: String,
    name: String,
    description: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
    created_by: String,
    version: i32,
    last_synced_at: Option<String>,
}

impl ProjectRow {
    fn into_project(
        self,
        contacts: Vec<Uuid>,
        tags: Vec<Uuid>,
        attachment_ids: Vec<Uuid>,
    ) -> Project {
        use core_domain::ProjectStatus;

        let status = match self.status.as_str() {
            "Active" => ProjectStatus::Active,
            "Completed" => ProjectStatus::Completed,
            "Archived" => ProjectStatus::Archived,
            "OnHold" => ProjectStatus::OnHold,
            _ => ProjectStatus::Active,
        };

        Project {
            id: Uuid::parse_str(&self.id).unwrap(),
            name: self.name,
            description: self.description,
            status,
            contacts,
            tags,
            attachment_ids,
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
        }
    }
}
