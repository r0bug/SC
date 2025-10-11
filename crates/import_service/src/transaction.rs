use crate::ImportError;
use chrono::Utc;
use sqlx::{Pool, Sqlite, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum TransactionState {
    NotStarted,
    InProgress,
    Committed,
    RolledBack,
    Failed(String),
}

pub struct ImportTransaction<'a> {
    pub id: Uuid,
    pub state: TransactionState,
    transaction: Option<Transaction<'a, Sqlite>>,
    imported_ids: Vec<Uuid>,
    changes_log: Vec<ChangeLog>,
    checkpoint_interval: usize,
    current_batch: usize,
}

#[derive(Debug, Clone)]
pub struct ChangeLog {
    pub batch_number: usize,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: Operation,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Insert,
    Update,
    Skip,
}

impl<'a> ImportTransaction<'a> {
    pub async fn new(pool: &'a Pool<Sqlite>) -> Result<Self, ImportError> {
        let transaction = pool.begin().await?;

        Ok(Self {
            id: Uuid::new_v4(),
            state: TransactionState::InProgress,
            transaction: Some(transaction),
            imported_ids: Vec::new(),
            changes_log: Vec::new(),
            checkpoint_interval: 100,
            current_batch: 0,
        })
    }

    pub fn add_imported_id(&mut self, id: Uuid) {
        self.imported_ids.push(id);
    }

    pub fn log_change(&mut self, entity_type: String, entity_id: Uuid, operation: Operation) {
        self.changes_log.push(ChangeLog {
            batch_number: self.current_batch,
            entity_type,
            entity_id,
            operation,
            timestamp: Utc::now(),
        });
    }

    pub async fn checkpoint(&mut self) -> Result<(), ImportError> {
        self.current_batch += 1;

        // In a real implementation, we would save checkpoint to database
        // allowing partial rollback to specific checkpoints
        if self.current_batch % self.checkpoint_interval == 0 {
            if let Some(tx) = &mut self.transaction {
                // Create savepoint
                sqlx::query("SAVEPOINT import_checkpoint")
                    .execute(&mut **tx)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn commit(mut self) -> Result<ImportSummary, ImportError> {
        if let Some(tx) = self.transaction.take() {
            tx.commit().await?;
            self.state = TransactionState::Committed;

            Ok(ImportSummary {
                transaction_id: self.id,
                total_processed: self.changes_log.len(),
                inserted: self
                    .changes_log
                    .iter()
                    .filter(|c| matches!(c.operation, Operation::Insert))
                    .count(),
                updated: self
                    .changes_log
                    .iter()
                    .filter(|c| matches!(c.operation, Operation::Update))
                    .count(),
                skipped: self
                    .changes_log
                    .iter()
                    .filter(|c| matches!(c.operation, Operation::Skip))
                    .count(),
                imported_ids: self.imported_ids,
            })
        } else {
            Err(ImportError::TransactionError(
                "No active transaction to commit".to_string(),
            ))
        }
    }

    pub async fn rollback(mut self) -> Result<(), ImportError> {
        if let Some(tx) = self.transaction.take() {
            tx.rollback().await?;
            self.state = TransactionState::RolledBack;
            Ok(())
        } else {
            Err(ImportError::TransactionError(
                "No active transaction to rollback".to_string(),
            ))
        }
    }

    pub async fn rollback_to_checkpoint(&mut self, checkpoint: usize) -> Result<(), ImportError> {
        if checkpoint > self.current_batch {
            return Err(ImportError::TransactionError(
                "Invalid checkpoint: future checkpoint specified".to_string(),
            ));
        }

        if let Some(tx) = &mut self.transaction {
            // Rollback to savepoint
            sqlx::query("ROLLBACK TO SAVEPOINT import_checkpoint")
                .execute(&mut **tx)
                .await?;

            // Remove changes after checkpoint
            self.changes_log.retain(|c| c.batch_number <= checkpoint);
            self.imported_ids.truncate(self.changes_log.len());
            self.current_batch = checkpoint;

            Ok(())
        } else {
            Err(ImportError::TransactionError(
                "No active transaction".to_string(),
            ))
        }
    }

    pub fn get_summary(&self) -> ImportProgress {
        ImportProgress {
            transaction_id: self.id,
            state: self.state.clone(),
            current_batch: self.current_batch,
            total_processed: self.changes_log.len(),
            inserted: self
                .changes_log
                .iter()
                .filter(|c| matches!(c.operation, Operation::Insert))
                .count(),
            updated: self
                .changes_log
                .iter()
                .filter(|c| matches!(c.operation, Operation::Update))
                .count(),
            skipped: self
                .changes_log
                .iter()
                .filter(|c| matches!(c.operation, Operation::Skip))
                .count(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportSummary {
    pub transaction_id: Uuid,
    pub total_processed: usize,
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub imported_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ImportProgress {
    pub transaction_id: Uuid,
    pub state: TransactionState,
    pub current_batch: usize,
    pub total_processed: usize,
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
}
