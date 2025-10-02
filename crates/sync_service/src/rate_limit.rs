// Rate limiting module using simple in-memory rate limiter
// This is a simplified implementation for Alpha - production should use Redis-backed rate limiting

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u64,
    pub burst_size: u32,
}

impl RateLimitConfig {
    /// Rate limit for authentication endpoints (strict)
    pub fn auth_routes() -> Self {
        Self {
            requests_per_second: 1,
            burst_size: 10,           // Allow burst of 10 requests
        }
    }

    /// Rate limit for attachment uploads (moderate)
    pub fn attachment_routes() -> Self {
        Self {
            requests_per_second: 1,
            burst_size: 100,          // Allow burst of 100 requests
        }
    }

    /// Rate limit for search operations (balanced)
    pub fn search_routes() -> Self {
        Self {
            requests_per_second: 1,
            burst_size: 30,           // Allow burst of 30 requests
        }
    }
}

#[derive(Debug)]
struct RateLimitEntry {
    tokens: u32,
    last_refill: Instant,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    state: Arc<Mutex<HashMap<IpAddr, RateLimitEntry>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn check_limit(&self, ip: IpAddr) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();

        let entry = state.entry(ip).or_insert(RateLimitEntry {
            tokens: self.config.burst_size,
            last_refill: now,
        });

        // Refill tokens based on time elapsed
        let elapsed = now.duration_since(entry.last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.config.requests_per_second as f64) as u32;

        if tokens_to_add > 0 {
            entry.tokens = (entry.tokens + tokens_to_add).min(self.config.burst_size);
            entry.last_refill = now;
        }

        // Check if we have tokens available
        if entry.tokens > 0 {
            entry.tokens -= 1;
            true
        } else {
            false
        }
    }

    pub async fn middleware(&self, req: Request, next: Next) -> Response {
        // Extract IP address from request
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<IpAddr>().ok())
            })
            .unwrap_or(IpAddr::from([127, 0, 0, 1])); // Default to localhost if no IP found

        // Check rate limit
        if !self.check_limit(ip).await {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(
                    "retry-after",
                    "60", // Retry after 60 seconds
                )],
                Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests. Please slow down and try again later.",
                    "retry_after": 60
                })),
            )
                .into_response();
        }

        next.run(req).await
    }
}

// Helper function to create rate limiter
pub fn create_rate_limiter(config: RateLimitConfig) -> RateLimiter {
    RateLimiter::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(config);
        let ip = "127.0.0.1".parse().unwrap();

        // Should allow 5 requests (burst)
        for _ in 0..5 {
            assert!(limiter.check_limit(ip).await);
        }

        // 6th request should be blocked
        assert!(!limiter.check_limit(ip).await);

        // Wait a bit for token refill
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Should allow more requests after refill
        assert!(limiter.check_limit(ip).await);
    }
}
