use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured logging with JSON format
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let format_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Check if JSON logging is requested
    if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(format_layer)
            .init();
    }

    tracing::info!("Structured logging initialized");
}

/// Initialize Prometheus metrics exporter
pub fn init_metrics() -> PrometheusHandle {
    let builder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            &[
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
        .unwrap()
        .set_buckets_for_metric(
            Matcher::Full("db_query_duration_seconds".to_string()),
            &[0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0],
        )
        .unwrap();

    let handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    tracing::info!("Prometheus metrics initialized");
    handle
}

/// Application metrics (used in metrics middleware and tests)
#[allow(dead_code)]
pub struct Metrics;

#[allow(dead_code)]
impl Metrics {
    /// Record HTTP request
    pub fn record_http_request(method: &str, path: &str, status: u16, duration: f64) {
        histogram!("http_request_duration_seconds",
            "method" => method.to_string(),
            "path" => path.to_string(),
            "status" => status.to_string()
        )
        .record(duration);

        counter!("http_requests_total",
            "method" => method.to_string(),
            "path" => path.to_string(),
            "status" => status.to_string()
        )
        .increment(1);
    }

    /// Record database query
    pub fn record_db_query(operation: &str, table: &str, duration: f64, success: bool) {
        histogram!("db_query_duration_seconds",
            "operation" => operation.to_string(),
            "table" => table.to_string()
        )
        .record(duration);

        counter!("db_queries_total",
            "operation" => operation.to_string(),
            "table" => table.to_string(),
            "success" => success.to_string()
        )
        .increment(1);
    }

    /// Record AI interaction
    pub fn record_ai_interaction(interaction_type: &str, cache_hit: bool, duration: f64) {
        histogram!("ai_interaction_duration_seconds",
            "type" => interaction_type.to_string(),
            "cache_hit" => cache_hit.to_string()
        )
        .record(duration);

        counter!("ai_interactions_total",
            "type" => interaction_type.to_string(),
            "cache_hit" => cache_hit.to_string()
        )
        .increment(1);

        if cache_hit {
            counter!("ai_cache_hits_total",
                "type" => interaction_type.to_string()
            )
            .increment(1);
        }
    }

    /// Record attachment upload
    pub fn record_attachment_upload(size_bytes: u64, content_type: &str, scan_status: &str) {
        histogram!("attachment_size_bytes",
            "content_type" => content_type.to_string()
        )
        .record(size_bytes as f64);

        counter!("attachments_uploaded_total",
            "content_type" => content_type.to_string(),
            "scan_status" => scan_status.to_string()
        )
        .increment(1);
    }

    /// Record WebSocket connections
    pub fn record_websocket_connection(connected: bool) {
        if connected {
            gauge!("websocket_connections_active").increment(1.0);
            counter!("websocket_connections_total").increment(1);
        } else {
            gauge!("websocket_connections_active").decrement(1.0);
        }
    }

    /// Record background worker job
    pub fn record_worker_job(job_type: &str, duration: f64, success: bool) {
        histogram!("worker_job_duration_seconds",
            "type" => job_type.to_string()
        )
        .record(duration);

        counter!("worker_jobs_total",
            "type" => job_type.to_string(),
            "success" => success.to_string()
        )
        .increment(1);
    }

    /// Update active connections gauge
    pub fn set_active_connections(count: u64) {
        gauge!("active_connections").set(count as f64);
    }

    /// Update database pool stats
    pub fn set_db_pool_stats(active: u32, idle: u32, max: u32) {
        gauge!("db_pool_connections_active").set(active as f64);
        gauge!("db_pool_connections_idle").set(idle as f64);
        gauge!("db_pool_connections_max").set(max as f64);
    }
}

/// Timer for measuring operation duration (used in metrics middleware and tests)
#[allow(dead_code)]
pub struct Timer {
    start: Instant,
}

#[allow(dead_code)]
impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_secs();
        assert!(elapsed >= 0.01, "Timer should measure at least 10ms");
    }

    #[test]
    fn test_metrics_recording() {
        // These just verify the API works, metrics go to global registry
        Metrics::record_http_request("GET", "/api/contacts", 200, 0.05);
        Metrics::record_db_query("SELECT", "contacts", 0.001, true);
        Metrics::record_ai_interaction("TagSuggestion", true, 0.1);
        Metrics::record_attachment_upload(1024, "application/pdf", "Clean");
        Metrics::record_worker_job("email", 0.5, true);
    }
}
