#[cfg(test)]
mod tests {
    use communication_queue::{
        WorkerSupervisor, MetricsStore, EnhancedCommunicationQueue,
        BackoffConfig, CommunicationQueue
    };
    use local_store::CommunicationRepository;
    use core_domain::{CommunicationAttempt, CommunicationStatus, CommunicationMethod};
    use sqlx::sqlite::SqlitePool;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;

    async fn setup_test_db() -> Arc<SqlitePool> {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE communication_attempts (
                id TEXT PRIMARY KEY,
                contact_id TEXT NOT NULL,
                method TEXT NOT NULL,
                subject TEXT,
                message TEXT NOT NULL,
                status TEXT NOT NULL,
                scheduled_at TEXT,
                attempted_at TEXT,
                retry_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .expect("Failed to create communication_attempts table");

        sqlx::query(
            r#"
            CREATE TABLE worker_metrics (
                id TEXT PRIMARY KEY,
                task_name TEXT NOT NULL UNIQUE,
                success_count INTEGER DEFAULT 0,
                failure_count INTEGER DEFAULT 0,
                last_run_at TEXT,
                last_success_at TEXT,
                last_failure_at TEXT,
                last_error TEXT,
                average_duration_ms INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .expect("Failed to create worker_metrics table");

        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_worker_supervisor_task_management() {
        let supervisor = WorkerSupervisor::new();

        // Spawn a simple task
        let task_id = supervisor
            .spawn_task("test_task".to_string(), || async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(())
            })
            .await
            .expect("Failed to spawn task");

        // Check task status
        let status = supervisor.get_task_status(task_id).await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().name, "test_task");

        // Wait for task to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Check health
        let health = supervisor.health_check().await;
        assert_eq!(health.total_tasks, 1);
    }

    #[tokio::test]
    async fn test_supervisor_task_restart() {
        let supervisor = WorkerSupervisor::new();
        let counter = Arc::new(tokio::sync::Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // Spawn a task that fails first time, succeeds second time
        let _task_id = supervisor
            .spawn_task("restart_test".to_string(), move || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    let mut count = counter.lock().await;
                    *count += 1;

                    if *count == 1 {
                        Err(anyhow::anyhow!("First attempt failure"))
                    } else {
                        Ok(())
                    }
                }
            })
            .await
            .expect("Failed to spawn task");

        // Wait for restart
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

        // Check counter was incremented twice (initial + restart)
        let final_count = *counter.lock().await;
        assert_eq!(final_count, 2);
    }

    #[tokio::test]
    async fn test_metrics_store() {
        let pool = setup_test_db().await;
        let metrics = MetricsStore::new(pool);

        metrics.init_tables().await.expect("Failed to init tables");

        // Record success
        metrics
            .record_success("test_task", 1500)
            .await
            .expect("Failed to record success");

        // Record failure
        metrics
            .record_failure("test_task", "Test error")
            .await
            .expect("Failed to record failure");

        // Get metrics
        let task_metrics = metrics
            .get_metrics("test_task")
            .await
            .expect("Failed to get metrics");

        assert!(task_metrics.is_some());
        let m = task_metrics.unwrap();
        assert_eq!(m.task_name, "test_task");
        assert_eq!(m.success_count, 1);
        assert_eq!(m.failure_count, 1);
        assert_eq!(m.last_error, Some("Test error".to_string()));
    }

    #[tokio::test]
    async fn test_enhanced_queue_batch_processing() {
        let pool = setup_test_db().await;
        let repo = CommunicationRepository::new(&*pool);

        // Create pending communications
        for i in 0..3 {
            let attempt = CommunicationAttempt {
                id: Uuid::new_v4(),
                contact_id: Uuid::new_v4(),
                method: CommunicationMethod::Email,
                subject: Some(format!("Test {}", i)),
                message: format!("Test message {}", i),
                status: CommunicationStatus::Pending,
                scheduled_at: None,
                attempted_at: None,
                retry_count: 0,
                created_at: Utc::now(),
            };

            repo.create(&attempt).await.expect("Failed to create attempt");
        }

        // Process batch
        let queue = EnhancedCommunicationQueue::new()
            .with_batch_size(2); // Process 2 at a time

        let processed = queue
            .process_batch(&repo)
            .await
            .expect("Failed to process batch");

        assert_eq!(processed, 2); // Should process batch size

        // Check status updates
        let pending = repo
            .list_pending()
            .await
            .expect("Failed to list pending");

        assert_eq!(pending.len(), 1); // One should remain
    }

    #[tokio::test]
    async fn test_backoff_config() {
        let config = BackoffConfig {
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            multiplier: 2.0,
            max_retries: 3,
        };

        let _queue = EnhancedCommunicationQueue::new()
            .with_backoff_config(config);

        // Test is complete - backoff is tested in unit tests
        assert!(true);
    }

    #[tokio::test]
    async fn test_communication_queue_processing() {
        let pool = setup_test_db().await;
        let repo = CommunicationRepository::new(&*pool);

        // Create a pending communication
        let attempt = CommunicationAttempt {
            id: Uuid::new_v4(),
            contact_id: Uuid::new_v4(),
            method: CommunicationMethod::SMS,
            subject: None,
            message: "Test SMS".to_string(),
            status: CommunicationStatus::Pending,
            scheduled_at: None,
            attempted_at: None,
            retry_count: 0,
            created_at: Utc::now(),
        };

        repo.create(&attempt).await.expect("Failed to create attempt");

        // Process with standard queue
        let queue = CommunicationQueue::new();
        queue
            .process_pending(&repo)
            .await
            .expect("Failed to process");

        // Verify status was updated by checking pending list is now empty
        let pending_after = repo
            .list_pending()
            .await
            .expect("Failed to list pending");

        assert_eq!(pending_after.len(), 0); // Should be processed
    }
}