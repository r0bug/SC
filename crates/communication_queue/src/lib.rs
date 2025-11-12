pub mod adapters;
pub mod email_ai_analyzer;
pub mod email_monitor;
pub mod queue;
pub mod queue_enhanced;
pub mod scheduler;
pub mod worker;

pub use email_ai_analyzer::{EmailAIAnalyzer, EmailAnalysis};
pub use email_monitor::{EmailMonitor, ImapConfig};
pub use queue::CommunicationQueue;
pub use queue_enhanced::{BackoffConfig, EnhancedCommunicationQueue, RateLimitConfig};
pub use scheduler::NagScheduler;
pub use worker::{
    CommunicationTask, HealthServer, MetricsStore, NagReminderTask, SuggestionTask, TaskHandle,
    WorkerSupervisor,
};
