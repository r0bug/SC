use core_domain::{CalendarEvent, DomainError, DomainResult, Reminder};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct CalendarEventRepository<'a> {
    pool: &'a Pool<Sqlite>,
}

impl<'a> CalendarEventRepository<'a> {
    pub fn new(pool: &'a Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, event: &CalendarEvent) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO calendar_events (id, title, description, start_time, end_time, all_day, location, recurrence, created_at, updated_at, created_by, version)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(event.id.to_string())
        .bind(&event.title)
        .bind(&event.description)
        .bind(event.start_time.to_rfc3339())
        .bind(event.end_time.to_rfc3339())
        .bind(event.all_day as i32)
        .bind(&event.location)
        .bind(event.recurrence.as_ref().map(|r| serde_json::to_string(r).unwrap()))
        .bind(event.created_at.to_rfc3339())
        .bind(event.updated_at.to_rfc3339())
        .bind(event.created_by.to_string())
        .bind(event.version)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        for contact_id in &event.contacts {
            sqlx::query("INSERT INTO event_contacts (event_id, contact_id) VALUES (?, ?)")
                .bind(event.id.to_string())
                .bind(contact_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        for reminder in &event.reminders {
            sqlx::query(
                "INSERT INTO event_reminders (id, event_id, minutes_before, method) VALUES (?, ?, ?, ?)"
            )
            .bind(reminder.id.to_string())
            .bind(event.id.to_string())
            .bind(reminder.minutes_before)
            .bind(serde_json::to_string(&reminder.method).unwrap())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: Uuid) -> DomainResult<CalendarEvent> {
        let row = sqlx::query_as::<_, CalendarEventRow>(
            "SELECT id, title, description, start_time, end_time, all_day, location, recurrence, created_at, updated_at, created_by, version
             FROM calendar_events WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::NotFound(format!("CalendarEvent {}", id)))?;

        let contacts = sqlx::query_scalar::<_, String>(
            "SELECT contact_id FROM event_contacts WHERE event_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        let reminder_rows = sqlx::query_as::<_, ReminderRow>(
            "SELECT id, minutes_before, method FROM event_reminders WHERE event_id = ?",
        )
        .bind(id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let reminders = reminder_rows.into_iter().map(|r| r.into()).collect();

        Ok(row.into_event(contacts, reminders, vec![]))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> DomainResult<Vec<CalendarEvent>> {
        let rows = sqlx::query_as::<_, CalendarEventRow>(
            "SELECT id, title, description, start_time, end_time, all_day, location, recurrence, created_at, updated_at, created_by, version
             FROM calendar_events ORDER BY start_time LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let _event_id = Uuid::parse_str(&row.id).unwrap();

            let contacts = sqlx::query_scalar::<_, String>(
                "SELECT contact_id FROM event_contacts WHERE event_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

            let reminder_rows = sqlx::query_as::<_, ReminderRow>(
                "SELECT id, minutes_before, method FROM event_reminders WHERE event_id = ?",
            )
            .bind(row.id.clone())
            .fetch_all(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

            let reminders = reminder_rows.into_iter().map(|r| r.into()).collect();

            events.push(row.into_event(contacts, reminders, vec![]));
        }

        Ok(events)
    }

    pub async fn update(&self, event: &CalendarEvent) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE calendar_events SET title = ?, description = ?, start_time = ?, end_time = ?, all_day = ?, location = ?, recurrence = ?, updated_at = ?, version = ?
             WHERE id = ?"
        )
        .bind(&event.title)
        .bind(&event.description)
        .bind(event.start_time.to_rfc3339())
        .bind(event.end_time.to_rfc3339())
        .bind(event.all_day as i32)
        .bind(&event.location)
        .bind(event.recurrence.as_ref().map(|r| serde_json::to_string(r).unwrap()))
        .bind(event.updated_at.to_rfc3339())
        .bind(event.version)
        .bind(event.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM event_contacts WHERE event_id = ?")
            .bind(event.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for contact_id in &event.contacts {
            sqlx::query("INSERT INTO event_contacts (event_id, contact_id) VALUES (?, ?)")
                .bind(event.id.to_string())
                .bind(contact_id.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        sqlx::query("DELETE FROM event_reminders WHERE event_id = ?")
            .bind(event.id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for reminder in &event.reminders {
            sqlx::query(
                "INSERT INTO event_reminders (id, event_id, minutes_before, method) VALUES (?, ?, ?, ?)"
            )
            .bind(reminder.id.to_string())
            .bind(event.id.to_string())
            .bind(reminder.minutes_before)
            .bind(serde_json::to_string(&reminder.method).unwrap())
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
        sqlx::query("DELETE FROM calendar_events WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_contact(&self, event_id: Uuid, contact_id: Uuid) -> DomainResult<()> {
        sqlx::query("INSERT OR IGNORE INTO event_contacts (event_id, contact_id) VALUES (?, ?)")
            .bind(event_id.to_string())
            .bind(contact_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_contact(&self, event_id: Uuid, contact_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM event_contacts WHERE event_id = ? AND contact_id = ?")
            .bind(event_id.to_string())
            .bind(contact_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn add_reminder(&self, event_id: Uuid, reminder: &Reminder) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO event_reminders (id, event_id, minutes_before, method) VALUES (?, ?, ?, ?)"
        )
        .bind(reminder.id.to_string())
        .bind(event_id.to_string())
        .bind(reminder.minutes_before)
        .bind(serde_json::to_string(&reminder.method).unwrap())
        .execute(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_reminder(&self, reminder_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM event_reminders WHERE id = ?")
            .bind(reminder_id.to_string())
            .execute(self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn list_by_creator(&self, creator_id: Uuid) -> DomainResult<Vec<CalendarEvent>> {
        let rows = sqlx::query_as::<_, CalendarEventRow>(
            "SELECT id, title, description, start_time, end_time, all_day, location,
             recurrence, created_at, updated_at, created_by, version
             FROM calendar_events WHERE created_by = ? ORDER BY start_time",
        )
        .bind(creator_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_id = Uuid::parse_str(&row.id).unwrap();
            let event = self.build_event_from_row(row, event_id).await?;
            events.push(event);
        }

        Ok(events)
    }

    pub async fn list_by_date_range(
        &self,
        creator_id: Uuid,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> DomainResult<Vec<CalendarEvent>> {
        let rows = sqlx::query_as::<_, CalendarEventRow>(
            "SELECT id, title, description, start_time, end_time, all_day, location,
             recurrence, created_at, updated_at, created_by, version
             FROM calendar_events
             WHERE created_by = ? AND start_time >= ? AND end_time <= ?
             ORDER BY start_time",
        )
        .bind(creator_id.to_string())
        .bind(start.to_rfc3339())
        .bind(end.to_rfc3339())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event_id = Uuid::parse_str(&row.id).unwrap();
            let event = self.build_event_from_row(row, event_id).await?;
            events.push(event);
        }

        Ok(events)
    }

    pub async fn list_by_contact(&self, contact_id: Uuid) -> DomainResult<Vec<CalendarEvent>> {
        let event_ids = sqlx::query_scalar::<_, String>(
            "SELECT event_id FROM event_contacts WHERE contact_id = ?",
        )
        .bind(contact_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut events = Vec::new();
        for event_id_str in event_ids {
            if let Ok(event_id) = Uuid::parse_str(&event_id_str) {
                if let Ok(event) = self.get_by_id(event_id).await {
                    events.push(event);
                }
            }
        }

        events.sort_by_key(|e| e.start_time);
        Ok(events)
    }

    async fn build_event_from_row(
        &self,
        row: CalendarEventRow,
        event_id: Uuid,
    ) -> DomainResult<CalendarEvent> {
        let contacts = sqlx::query_scalar::<_, String>(
            "SELECT contact_id FROM event_contacts WHERE event_id = ?",
        )
        .bind(event_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        let reminders = sqlx::query_as::<_, ReminderRow>(
            "SELECT id, minutes_before, method FROM event_reminders WHERE event_id = ?",
        )
        .bind(event_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .map(|r| r.into())
        .collect();

        let attachment_ids = sqlx::query_scalar::<_, String>(
            "SELECT attachment_id FROM event_attachments WHERE event_id = ?",
        )
        .bind(event_id.to_string())
        .fetch_all(self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

        Ok(row.into_event(contacts, reminders, attachment_ids))
    }
}

#[derive(sqlx::FromRow)]
struct CalendarEventRow {
    id: String,
    title: String,
    description: Option<String>,
    start_time: String,
    end_time: String,
    all_day: i32,
    location: Option<String>,
    recurrence: Option<String>,
    created_at: String,
    updated_at: String,
    created_by: String,
    version: i32,
}

impl CalendarEventRow {
    fn into_event(
        self,
        contacts: Vec<Uuid>,
        reminders: Vec<Reminder>,
        attachment_ids: Vec<Uuid>,
    ) -> CalendarEvent {
        CalendarEvent {
            id: Uuid::parse_str(&self.id).unwrap(),
            title: self.title,
            description: self.description,
            start_time: chrono::DateTime::parse_from_rfc3339(&self.start_time)
                .unwrap()
                .with_timezone(&chrono::Utc),
            end_time: chrono::DateTime::parse_from_rfc3339(&self.end_time)
                .unwrap()
                .with_timezone(&chrono::Utc),
            all_day: self.all_day != 0,
            contacts,
            location: self.location,
            recurrence: self.recurrence.and_then(|s| serde_json::from_str(&s).ok()),
            reminders,
            attachment_ids,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created_by: Uuid::parse_str(&self.created_by).unwrap(),
            version: self.version,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ReminderRow {
    id: String,
    minutes_before: i32,
    method: String,
}

impl From<ReminderRow> for Reminder {
    fn from(row: ReminderRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap(),
            minutes_before: row.minutes_before,
            method: serde_json::from_str(&row.method).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use core_domain::ReminderMethod;
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
    async fn test_create_and_get_event() {
        let pool = setup_test_db().await;
        let repo = CalendarEventRepository::new(&pool);

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

        let event = CalendarEvent {
            id: Uuid::new_v4(),
            title: "Test Meeting".to_string(),
            description: Some("A test event".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            all_day: false,
            contacts: vec![],
            location: Some("Office".to_string()),
            recurrence: None,
            reminders: vec![],
            attachment_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
            version: 1,
        };

        repo.create(&event).await.unwrap();

        let retrieved = repo.get_by_id(event.id).await.unwrap();
        assert_eq!(retrieved.title, event.title);
        assert_eq!(retrieved.location, event.location);
    }

    #[tokio::test]
    async fn test_add_and_remove_reminder() {
        let pool = setup_test_db().await;
        let repo = CalendarEventRepository::new(&pool);

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

        let event = CalendarEvent {
            id: Uuid::new_v4(),
            title: "Test Meeting".to_string(),
            description: None,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            all_day: false,
            contacts: vec![],
            location: None,
            recurrence: None,
            reminders: vec![],
            attachment_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: user_id,
            version: 1,
        };

        repo.create(&event).await.unwrap();

        let reminder = Reminder {
            id: Uuid::new_v4(),
            minutes_before: 15,
            method: ReminderMethod::Email,
        };

        repo.add_reminder(event.id, &reminder).await.unwrap();

        let retrieved = repo.get_by_id(event.id).await.unwrap();
        assert_eq!(retrieved.reminders.len(), 1);
        assert_eq!(retrieved.reminders[0].minutes_before, 15);

        repo.remove_reminder(reminder.id).await.unwrap();

        let retrieved = repo.get_by_id(event.id).await.unwrap();
        assert_eq!(retrieved.reminders.len(), 0);
    }
}
