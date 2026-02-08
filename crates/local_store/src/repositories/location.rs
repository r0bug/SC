use core_domain::{DomainError, DomainResult, Location};
use crate::db::DbPool;
use uuid::Uuid;

pub struct LocationRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> LocationRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, location: &Location) -> DomainResult<()> {
        let metadata_str = serde_json::to_string(&location.metadata)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize metadata: {}", e)))?;

        sqlx::query(
            "INSERT INTO locations (id, name, address, city, state, zip, coordinates_lat, coordinates_lng, metadata, created_at, updated_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(location.id.to_string())
        .bind(&location.name)
        .bind(&location.address)
        .bind(&location.city)
        .bind(&location.state)
        .bind(&location.zip)
        .bind(location.coordinates_lat)
        .bind(location.coordinates_lng)
        .bind(&metadata_str)
        .bind(location.created_at.to_rfc3339())
        .bind(location.updated_at.to_rfc3339())
        .bind(location.created_by.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to insert location: {}", e)))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<Location> {
        let row = sqlx::query_as::<_, LocationRow>(
            "SELECT id, name, address, city, state, zip, coordinates_lat, coordinates_lng, metadata, created_at, updated_at, created_by
             FROM locations WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("Location {}", id)))?;

        Ok(row.into_location())
    }

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<Location>> {
        let rows = sqlx::query_as::<_, LocationRow>(
            "SELECT id, name, address, city, state, zip, coordinates_lat, coordinates_lng, metadata, created_at, updated_at, created_by
             FROM locations ORDER BY name LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_location()).collect())
    }

    pub async fn list_by_creator(&self, creator_id: Uuid) -> DomainResult<Vec<Location>> {
        let rows = sqlx::query_as::<_, LocationRow>(
            "SELECT id, name, address, city, state, zip, coordinates_lat, coordinates_lng, metadata, created_at, updated_at, created_by
             FROM locations WHERE created_by = ? ORDER BY name",
        )
        .bind(creator_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_location()).collect())
    }

    pub async fn update(&self, location: &Location) -> DomainResult<()> {
        let metadata_str = serde_json::to_string(&location.metadata)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize metadata: {}", e)))?;

        sqlx::query(
            "UPDATE locations SET name = ?, address = ?, city = ?, state = ?, zip = ?, coordinates_lat = ?, coordinates_lng = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&location.name)
        .bind(&location.address)
        .bind(&location.city)
        .bind(&location.state)
        .bind(&location.zip)
        .bind(location.coordinates_lat)
        .bind(location.coordinates_lng)
        .bind(&metadata_str)
        .bind(location.updated_at.to_rfc3339())
        .bind(location.id.to_string())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(format!("Failed to update location: {}", e)))?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM locations WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to delete location: {}", e)))?;

        Ok(())
    }

    pub async fn search(&self, query: &str) -> DomainResult<Vec<Location>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, LocationRow>(
            "SELECT id, name, address, city, state, zip, coordinates_lat, coordinates_lng, metadata, created_at, updated_at, created_by
             FROM locations
             WHERE name LIKE ? OR address LIKE ? OR city LIKE ?
             ORDER BY name
             LIMIT 50",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_location()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct LocationRow {
    id: String,
    name: String,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
    coordinates_lat: Option<f64>,
    coordinates_lng: Option<f64>,
    metadata: String,
    created_at: String,
    updated_at: String,
    created_by: String,
}

impl LocationRow {
    fn into_location(self) -> Location {
        Location {
            id: Uuid::parse_str(&self.id).unwrap(),
            name: self.name,
            address: self.address,
            city: self.city,
            state: self.state,
            zip: self.zip,
            coordinates_lat: self.coordinates_lat,
            coordinates_lng: self.coordinates_lng,
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

    fn make_location(user_id: Uuid) -> Location {
        Location {
            id: Uuid::new_v4(),
            name: "Test Location".to_string(),
            address: Some("123 Main St".to_string()),
            city: Some("Springfield".to_string()),
            state: Some("IL".to_string()),
            zip: Some("62701".to_string()),
            coordinates_lat: Some(39.7817),
            coordinates_lng: Some(-89.6501),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_location() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let location = make_location(user_id);
        repo.create(&location).await.unwrap();

        let retrieved = repo.get_by_id(location.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Location");
        assert_eq!(retrieved.address.as_deref(), Some("123 Main St"));
        assert_eq!(retrieved.city.as_deref(), Some("Springfield"));
        assert_eq!(retrieved.state.as_deref(), Some("IL"));
        assert_eq!(retrieved.zip.as_deref(), Some("62701"));
        assert_eq!(retrieved.coordinates_lat, Some(39.7817));
        assert_eq!(retrieved.coordinates_lng, Some(-89.6501));
        assert_eq!(retrieved.created_by, user_id);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);

        let result = repo.get_by_id(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_locations() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let mut loc1 = make_location(user_id);
        loc1.name = "Alpha Location".to_string();
        let mut loc2 = make_location(user_id);
        loc2.name = "Beta Location".to_string();

        repo.create(&loc1).await.unwrap();
        repo.create(&loc2).await.unwrap();

        let all = repo.list(10, 0).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "Alpha Location");
        assert_eq!(all[1].name, "Beta Location");

        let page = repo.list(1, 1).await.unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].name, "Beta Location");
    }

    #[tokio::test]
    async fn test_list_by_creator() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let location = make_location(user_id);
        repo.create(&location).await.unwrap();

        let results = repo.list_by_creator(user_id).await.unwrap();
        assert_eq!(results.len(), 1);

        let other_user = Uuid::new_v4();
        let empty = repo.list_by_creator(other_user).await.unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[tokio::test]
    async fn test_update_location() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let mut location = make_location(user_id);
        repo.create(&location).await.unwrap();

        location.name = "Updated Location".to_string();
        location.city = Some("Chicago".to_string());
        location.coordinates_lat = Some(41.8781);
        location.coordinates_lng = Some(-87.6298);
        location.updated_at = Utc::now();

        repo.update(&location).await.unwrap();

        let retrieved = repo.get_by_id(location.id).await.unwrap();
        assert_eq!(retrieved.name, "Updated Location");
        assert_eq!(retrieved.city.as_deref(), Some("Chicago"));
        assert_eq!(retrieved.coordinates_lat, Some(41.8781));
    }

    #[tokio::test]
    async fn test_delete_location() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let location = make_location(user_id);
        repo.create(&location).await.unwrap();

        repo.delete(location.id).await.unwrap();

        let result = repo.get_by_id(location.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_locations() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let mut loc1 = make_location(user_id);
        loc1.name = "Downtown Warehouse".to_string();
        loc1.address = Some("100 Market St".to_string());
        loc1.city = Some("Springfield".to_string());

        let mut loc2 = make_location(user_id);
        loc2.name = "Uptown Office".to_string();
        loc2.address = Some("200 Oak Ave".to_string());
        loc2.city = Some("Chicago".to_string());

        let mut loc3 = make_location(user_id);
        loc3.name = "Rural Barn".to_string();
        loc3.address = Some("999 Country Rd".to_string());
        loc3.city = Some("Springfield".to_string());

        repo.create(&loc1).await.unwrap();
        repo.create(&loc2).await.unwrap();
        repo.create(&loc3).await.unwrap();

        // Search by name
        let results = repo.search("Warehouse").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Downtown Warehouse");

        // Search by city
        let results = repo.search("Springfield").await.unwrap();
        assert_eq!(results.len(), 2);

        // Search by address
        let results = repo.search("Oak Ave").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Uptown Office");

        // Search with no matches
        let results = repo.search("Nonexistent").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_create_location_with_null_optionals() {
        let pool = setup_test_db().await;
        let repo = LocationRepository::new(&pool);
        let user_id = create_test_user(&pool).await;

        let location = Location {
            id: Uuid::new_v4(),
            name: "Minimal Location".to_string(),
            address: None,
            city: None,
            state: None,
            zip: None,
            coordinates_lat: None,
            coordinates_lng: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
        };

        repo.create(&location).await.unwrap();

        let retrieved = repo.get_by_id(location.id).await.unwrap();
        assert_eq!(retrieved.name, "Minimal Location");
        assert!(retrieved.address.is_none());
        assert!(retrieved.city.is_none());
        assert!(retrieved.state.is_none());
        assert!(retrieved.zip.is_none());
        assert!(retrieved.coordinates_lat.is_none());
        assert!(retrieved.coordinates_lng.is_none());
    }
}
