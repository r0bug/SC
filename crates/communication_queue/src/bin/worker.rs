use anyhow::Result;
use communication_queue::{
    CommunicationTask, HealthServer, MetricsStore, NagReminderTask, SuggestionTask,
    WorkerSupervisor,
};
use local_store::LocalStore;
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🚀 SagensContact Worker starting...");

    // Database setup
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/contacts.db".to_string());

    info!("📂 Connecting to database: {}", database_url);
    let store = Arc::new(LocalStore::new(&database_url).await?);

    // Sync service URL for callbacks (optional)
    let sync_service_url = std::env::var("SYNC_SERVICE_URL").ok();
    if let Some(ref url) = sync_service_url {
        info!("🔗 Will send callbacks to sync service at: {}", url);
    }

    // Initialize metrics store
    let metrics_store = Arc::new(MetricsStore::new(Arc::new(store.pool().clone())));
    metrics_store.init_tables().await?;
    info!("📊 Metrics store initialized");

    // Create supervisor
    let supervisor = Arc::new(WorkerSupervisor::new());
    info!("👨‍💼 Worker supervisor created");

    // Spawn communication processing task
    let comm_store = Arc::clone(&store);
    let comm_sync_url = sync_service_url.clone();
    let comm_metrics = Arc::clone(&metrics_store);
    let comm_id = supervisor
        .spawn_task("communication_processor".to_string(), move || {
            let store = Arc::clone(&comm_store);
            let url = comm_sync_url.clone();
            let metrics = Arc::clone(&comm_metrics);
            async move {
                let start = std::time::Instant::now();
                let task = CommunicationTask::new(store, url);

                match task.run().await {
                    Ok(_) => {
                        let duration = start.elapsed().as_millis() as i64;
                        metrics
                            .record_success("communication_processor", duration)
                            .await?;
                        Ok(())
                    }
                    Err(e) => {
                        metrics
                            .record_failure("communication_processor", &e.to_string())
                            .await?;
                        Err(e)
                    }
                }
            }
        })
        .await?;
    info!("✅ Communication processor task started (id: {})", comm_id);

    // Spawn nag reminder task
    let nag_metrics = Arc::clone(&metrics_store);
    let nag_id = supervisor
        .spawn_task("nag_reminder".to_string(), move || {
            let metrics = Arc::clone(&nag_metrics);
            async move {
                let start = std::time::Instant::now();
                let task = NagReminderTask::new().await?;

                match task.run().await {
                    Ok(_) => {
                        let duration = start.elapsed().as_millis() as i64;
                        metrics.record_success("nag_reminder", duration).await?;
                        Ok(())
                    }
                    Err(e) => {
                        metrics
                            .record_failure("nag_reminder", &e.to_string())
                            .await?;
                        Err(e)
                    }
                }
            }
        })
        .await?;
    info!("✅ Nag reminder task started (id: {})", nag_id);

    // Spawn suggestion generator task
    let suggest_store = Arc::clone(&store);
    let suggest_metrics = Arc::clone(&metrics_store);
    let suggest_id = supervisor
        .spawn_task("suggestion_generator".to_string(), move || {
            let store = Arc::clone(&suggest_store);
            let metrics = Arc::clone(&suggest_metrics);
            async move {
                let start = std::time::Instant::now();
                let task = SuggestionTask::new(store);

                match task.run().await {
                    Ok(_) => {
                        let duration = start.elapsed().as_millis() as i64;
                        metrics
                            .record_success("suggestion_generator", duration)
                            .await?;
                        Ok(())
                    }
                    Err(e) => {
                        metrics
                            .record_failure("suggestion_generator", &e.to_string())
                            .await?;
                        Err(e)
                    }
                }
            }
        })
        .await?;
    info!("✅ Suggestion generator task started (id: {})", suggest_id);

    // Start health server in background
    let health_supervisor = Arc::clone(&supervisor);
    let health_metrics = Arc::clone(&metrics_store);
    tokio::spawn(async move {
        let health_port: u16 = std::env::var("HEALTH_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse()
            .unwrap_or(9090);

        let health_server =
            HealthServer::new(health_supervisor, health_metrics).with_port(health_port);

        info!("🏥 Starting health server on port {}", health_port);

        if let Err(e) = health_server.start().await {
            error!("Health server failed: {}", e);
        }
    });

    info!("✅ Worker fully initialized with {} tasks", 3);
    info!("📋 Tasks will run continuously with automatic restart on failure");
    info!("🏥 Health check available at http://localhost:9090/health/worker");
    info!("⚠️  NOTE: All email/SMS sends are MOCKED in this alpha release");

    // Keep the main task alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        // Log health status periodically
        let health = supervisor.health_check().await;
        info!(
            "🔍 Health check - Tasks: {} total, {} running, {} failed",
            health.total_tasks, health.running_tasks, health.failed_tasks
        );

        if health.failed_tasks > 0 {
            error!(
                "⚠️  {} tasks have failed and exceeded max restarts",
                health.failed_tasks
            );
        }
    }
}
