use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: Option<String>,
    pub api_url: String,
    pub user_id: Option<String>,
    pub email: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token: None,
            api_url: "http://localhost:3000".to_string(),
            user_id: None,
            email: None,
        }
    }
}

impl AuthConfig {
    pub fn config_path() -> PathBuf {
        let mut path = config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sagenscontact");
        path.push("config.json");
        path
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let contents = fs::read_to_string(&path).context("Failed to read auth config")?;
            serde_json::from_str(&contents).context("Failed to parse auth config")
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let contents =
            serde_json::to_string_pretty(self).context("Failed to serialize auth config")?;
        fs::write(&path, contents).context("Failed to write auth config")?;
        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn clear(&mut self) {
        self.token = None;
        self.user_id = None;
        self.email = None;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
}

pub async fn login(email: String, password: String, api_url: &str) -> Result<AuthResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/login", api_url))
        .json(&LoginRequest { email, password })
        .send()
        .await
        .context("Failed to send login request")?;

    if !response.status().is_success() {
        anyhow::bail!("Login failed: {}", response.status());
    }

    response
        .json::<AuthResponse>()
        .await
        .context("Failed to parse login response")
}

pub async fn signup(
    email: String,
    password: String,
    name: String,
    api_url: &str,
) -> Result<AuthResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/signup", api_url))
        .json(&SignupRequest {
            email,
            password,
            name,
        })
        .send()
        .await
        .context("Failed to send signup request")?;

    if !response.status().is_success() {
        anyhow::bail!("Signup failed: {}", response.status());
    }

    response
        .json::<AuthResponse>()
        .await
        .context("Failed to parse signup response")
}
