use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

pub struct NagScheduler {
    scheduler: JobScheduler,
}

impl NagScheduler {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self { scheduler })
    }

    pub async fn schedule_nag_check(&mut self) -> Result<()> {
        let job = Job::new_async("0 */15 * * * *", |_uuid, _l| {
            Box::pin(async move {
                info!("[NAG] Checking for pending nagging reminders...");
            })
        })?;

        self.scheduler.add(job).await?;
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting nag scheduler");
        self.scheduler.start().await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down nag scheduler");
        self.scheduler.shutdown().await?;
        Ok(())
    }
}
