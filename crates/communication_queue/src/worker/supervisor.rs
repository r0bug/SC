use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TaskHandle {
    pub id: Uuid,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub status: TaskStatus,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Running,
    Stopped,
    Failed,
    Restarting,
}

pub struct WorkerSupervisor {
    tasks: Arc<RwLock<HashMap<Uuid, TaskHandle>>>,
    handles: Arc<Mutex<HashMap<Uuid, JoinHandle<()>>>>,
    max_restarts: u32,
    restart_delay: Duration,
}

impl Default for WorkerSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerSupervisor {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
            max_restarts: 5,
            restart_delay: Duration::from_secs(5),
        }
    }

    pub async fn spawn_task<F, Fut>(&self, name: String, mut task_fn: F) -> Result<Uuid>
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let task_id = Uuid::new_v4();
        let task_handle = TaskHandle {
            id: task_id,
            name: name.clone(),
            started_at: Utc::now(),
            status: TaskStatus::Running,
            restart_count: 0,
            last_error: None,
        };

        // Store task metadata
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task_handle.clone());
        }

        // Spawn the task with supervision
        let tasks_ref = Arc::clone(&self.tasks);
        let handles_ref = Arc::clone(&self.handles);
        let max_restarts = self.max_restarts;
        let restart_delay = self.restart_delay;

        let handle = tokio::spawn(async move {
            let mut restart_count = 0;

            loop {
                info!("Starting task: {} (id: {})", name, task_id);

                // Run the task
                match task_fn().await {
                    Ok(_) => {
                        info!("Task {} completed successfully", name);
                        break;
                    }
                    Err(e) => {
                        error!("Task {} failed: {}", name, e);

                        // Update task status
                        {
                            let mut tasks = tasks_ref.write().await;
                            if let Some(task) = tasks.get_mut(&task_id) {
                                task.status = TaskStatus::Failed;
                                task.last_error = Some(e.to_string());
                                task.restart_count = restart_count;
                            }
                        }

                        // Check if we should restart
                        if restart_count >= max_restarts {
                            error!("Task {} exceeded max restarts ({})", name, max_restarts);
                            break;
                        }

                        restart_count += 1;
                        warn!(
                            "Restarting task {} (attempt {}/{})",
                            name, restart_count, max_restarts
                        );

                        // Update status to restarting
                        {
                            let mut tasks = tasks_ref.write().await;
                            if let Some(task) = tasks.get_mut(&task_id) {
                                task.status = TaskStatus::Restarting;
                            }
                        }

                        // Wait before restart
                        sleep(restart_delay).await;

                        // Update status back to running
                        {
                            let mut tasks = tasks_ref.write().await;
                            if let Some(task) = tasks.get_mut(&task_id) {
                                task.status = TaskStatus::Running;
                                task.restart_count = restart_count;
                            }
                        }
                    }
                }
            }

            // Mark task as stopped when done
            {
                let mut tasks = tasks_ref.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.status = TaskStatus::Stopped;
                }
            }

            // Remove handle when done
            {
                let mut handles = handles_ref.lock().await;
                handles.remove(&task_id);
            }
        });

        // Store the join handle
        {
            let mut handles = self.handles.lock().await;
            handles.insert(task_id, handle);
        }

        Ok(task_id)
    }

    pub async fn stop_task(&self, task_id: Uuid) -> Result<()> {
        // Cancel the task handle
        {
            let mut handles = self.handles.lock().await;
            if let Some(handle) = handles.remove(&task_id) {
                handle.abort();
                info!("Task {} stopped", task_id);
            }
        }

        // Update task status
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Stopped;
            }
        }

        Ok(())
    }

    pub async fn get_task_status(&self, task_id: Uuid) -> Option<TaskHandle> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<TaskHandle> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn health_check(&self) -> HealthReport {
        let tasks = self.tasks.read().await;

        let total_tasks = tasks.len();
        let running_tasks = tasks
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .count();
        let failed_tasks = tasks
            .values()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();

        HealthReport {
            healthy: failed_tasks == 0 && running_tasks > 0,
            total_tasks,
            running_tasks,
            failed_tasks,
            tasks: tasks.values().cloned().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthReport {
    pub healthy: bool,
    pub total_tasks: usize,
    pub running_tasks: usize,
    pub failed_tasks: usize,
    pub tasks: Vec<TaskHandle>,
}
