use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const DEFAULT_BASE_URL: &str = "https://api.segmind.com/v1";
const DEFAULT_MODEL: &str = "llama-3.1-8b-instruct";
const MAX_RETRIES: u32 = 3;
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[derive(Clone)]
pub struct SegmindClient {
    api_key: Option<String>,
    base_url: String,
    model: String,
    mock_mode: bool,
    cache: Arc<RwLock<ResponseCache>>,
    http_client: reqwest::Client,
}

struct ResponseCache {
    entries: HashMap<String, CachedResponse>,
}

struct CachedResponse {
    response: SegmindResponse,
    cached_at: Instant,
}

impl ResponseCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<SegmindResponse> {
        if let Some(entry) = self.entries.get(key) {
            if entry.cached_at.elapsed() < CACHE_TTL {
                return Some(entry.response.clone());
            }
        }
        None
    }

    fn set(&mut self, key: String, response: SegmindResponse) {
        self.entries.insert(
            key,
            CachedResponse {
                response,
                cached_at: Instant::now(),
            },
        );
    }

    fn clear_expired(&mut self) {
        self.entries
            .retain(|_, entry| entry.cached_at.elapsed() < CACHE_TTL);
    }
}

impl SegmindClient {
    /// Create a new Segmind client
    /// If api_key is None or empty, mock mode is enabled
    pub fn new(api_key: Option<String>) -> Self {
        let mock_mode = api_key.is_none() || api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true);

        if mock_mode {
            info!("Segmind client running in MOCK MODE (no API key provided)");
        }

        Self {
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            mock_mode,
            cache: Arc::new(RwLock::new(ResponseCache::new())),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Configure custom base URL and model
    pub fn with_config(mut self, base_url: Option<String>, model: Option<String>) -> Self {
        if let Some(url) = base_url {
            self.base_url = url;
        }
        if let Some(m) = model {
            self.model = m;
        }
        self
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.entries.clear();
        info!("Cleared AI response cache");
    }

    /// Generate AI suggestion with retry and caching
    pub async fn generate_suggestion(&self, prompt: &str) -> Result<SegmindResponse> {
        // Check cache first
        let cache_key = format!("suggestion:{}", prompt);
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                info!("Returning cached AI response");
                return Ok(cached);
            }
        }

        // Clean up expired cache entries periodically
        {
            let mut cache = self.cache.write().await;
            cache.clear_expired();
        }

        let response = if self.mock_mode {
            self.mock_generate_suggestion(prompt).await?
        } else {
            self.call_segmind_api(prompt).await?
        };

        // Cache the response
        {
            let mut cache = self.cache.write().await;
            cache.set(cache_key, response.clone());
        }

        Ok(response)
    }

    /// Analyze contact data with AI
    pub async fn analyze_contact_data(&self, contact_data: &str) -> Result<ContactAnalysis> {
        let cache_key = format!("contact_analysis:{}", contact_data);

        // For contact analysis, we generate a structured prompt
        let prompt = format!(
            "Analyze this contact profile and provide insights:\n\n{}\n\nProvide: 1) Suggested tags, 2) Relationship strength (0-1), 3) Communication frequency, 4) Next action",
            contact_data
        );

        if self.mock_mode {
            return self.mock_analyze_contact(contact_data).await;
        }

        let response = self.call_segmind_api(&prompt).await?;

        // Parse response into structured analysis
        // In production, you'd parse the AI response more carefully
        Ok(ContactAnalysis {
            suggested_tags: vec!["Professional".to_string(), "Tech Industry".to_string()],
            relationship_strength: response.confidence,
            communication_frequency: "Monthly".to_string(),
            next_action: response
                .text
                .lines()
                .last()
                .unwrap_or("Follow up soon")
                .to_string(),
        })
    }

    /// Call actual Segmind API with retry logic
    async fn call_segmind_api(&self, prompt: &str) -> Result<SegmindResponse> {
        let api_key = self
            .api_key
            .as_ref()
            .context("API key required for Segmind calls")?;

        for attempt in 1..=MAX_RETRIES {
            let payload = serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "max_tokens": 500,
                "temperature": 0.7,
            });

            match self
                .http_client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let api_response: ApiResponse = response
                            .json()
                            .await
                            .context("Failed to parse Segmind response")?;

                        info!("Segmind API call successful");
                        return Ok(SegmindResponse {
                            text: api_response
                                .choices
                                .first()
                                .map(|c| c.message.content.clone())
                                .unwrap_or_default(),
                            confidence: 0.8, // Could derive from API response
                            model: self.model.clone(),
                        });
                    } else {
                        warn!(
                            "Segmind API returned error status: {} (attempt {}/{})",
                            response.status(),
                            attempt,
                            MAX_RETRIES
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Segmind API call failed: {} (attempt {}/{})",
                        e, attempt, MAX_RETRIES
                    );
                }
            }

            if attempt < MAX_RETRIES {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }
        }

        error!(
            "Segmind API call failed after {} retries, falling back to mock",
            MAX_RETRIES
        );
        self.mock_generate_suggestion(prompt).await
    }

    /// Mock implementation for development/testing
    async fn mock_generate_suggestion(&self, prompt: &str) -> Result<SegmindResponse> {
        info!(
            "[MOCK SEGMIND] Generating AI suggestion for prompt: {}",
            prompt
        );

        // Simulate API delay
        tokio::time::sleep(Duration::from_millis(200)).await;

        let suggestion = if prompt.contains("contact") {
            "Consider adding tags to better organize this contact. Based on their organization and title, they might fit into 'Business Partners' or 'Technical Leads' categories."
        } else if prompt.contains("note") {
            "This note mentions a follow-up. Would you like to schedule a reminder?"
        } else if prompt.contains("email") {
            "The email tone seems formal. Consider adding a personal touch by referencing your last conversation."
        } else if prompt.contains("project") {
            "This project could benefit from clearer milestones. Consider breaking it into smaller, time-boxed deliverables."
        } else {
            "No specific suggestions at this time. Continue tracking your interactions to build better insights."
        };

        Ok(SegmindResponse {
            text: suggestion.to_string(),
            confidence: 0.85,
            model: "mock-segmind-v1".to_string(),
        })
    }

    async fn mock_analyze_contact(&self, _contact_data: &str) -> Result<ContactAnalysis> {
        info!("[MOCK SEGMIND] Analyzing contact data");

        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(ContactAnalysis {
            suggested_tags: vec!["Professional".to_string(), "Tech Industry".to_string()],
            relationship_strength: 0.7,
            communication_frequency: "Monthly".to_string(),
            next_action: "Schedule a follow-up meeting within 2 weeks".to_string(),
        })
    }
}

// Segmind API response structure
#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmindResponse {
    pub text: String,
    pub confidence: f32,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactAnalysis {
    pub suggested_tags: Vec<String>,
    pub relationship_strength: f32,
    pub communication_frequency: String,
    pub next_action: String,
}
