use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_REPO: &str = "r0bug/SC";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub struct UpdateChecker {
    repo: String,
    current_version: String,
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new(GITHUB_REPO.to_string(), CURRENT_VERSION.to_string())
    }
}

impl UpdateChecker {
    pub fn new(repo: String, current_version: String) -> Self {
        Self {
            repo,
            current_version,
        }
    }

    /// Check for updates by querying GitHub releases API
    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        let url = format!("{}/repos/{}/releases/latest", GITHUB_API_BASE, self.repo);

        let client = reqwest::Client::builder()
            .user_agent("sagenscontact-updater")
            .build()?;

        let response = client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(UpdateInfo {
                current_version: self.current_version.clone(),
                latest_version: self.current_version.clone(),
                update_available: false,
                release_url: None,
                release_notes: None,
                download_url: None,
                published_at: None,
            });
        }

        let release: GitHubRelease = response.json().await?;

        // Remove 'v' prefix if present
        let latest_version = release.tag_name.trim_start_matches('v').to_string();
        let update_available = self.is_newer_version(&latest_version)?;

        // Find appropriate binary for current platform
        let platform = self.get_platform_name();
        let download_url = release.assets
            .iter()
            .find(|asset| asset.name.contains(&platform))
            .map(|asset| asset.browser_download_url.clone());

        Ok(UpdateInfo {
            current_version: self.current_version.clone(),
            latest_version,
            update_available,
            release_url: Some(release.html_url),
            release_notes: Some(release.body),
            download_url,
            published_at: Some(release.published_at),
        })
    }

    /// Compare semantic versions
    fn is_newer_version(&self, latest: &str) -> Result<bool> {
        let current_parts = self.parse_version(&self.current_version)?;
        let latest_parts = self.parse_version(latest)?;

        Ok(latest_parts > current_parts)
    }

    fn parse_version(&self, version: &str) -> Result<(u32, u32, u32)> {
        // Strip pre-release identifiers (e.g., -alpha.3, -beta.1)
        let version_clean = version.split('-').next().unwrap_or(version);
        let parts: Vec<&str> = version_clean.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid version format: {}", version));
        }

        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;
        let patch = parts[2].parse::<u32>()?;

        Ok((major, minor, patch))
    }

    fn get_platform_name(&self) -> String {
        #[cfg(target_os = "linux")]
        return "ubuntu-latest".to_string();

        #[cfg(target_os = "macos")]
        return "macos-latest".to_string();

        #[cfg(target_os = "windows")]
        return "windows-latest".to_string();

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return "unknown".to_string();
    }

    /// Download update binary
    pub async fn download_update(&self, download_url: &str, dest_path: &Path) -> Result<()> {
        let client = reqwest::Client::builder()
            .user_agent("sagenscontact-updater")
            .build()?;

        let response = client.get(download_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download update: HTTP {}", response.status()));
        }

        let bytes = response.bytes().await?;
        fs::write(dest_path, bytes)?;

        Ok(())
    }

    /// Apply update by replacing current binary
    pub fn apply_update(&self, update_path: &Path) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("bak");

        // Create backup of current binary
        if current_exe.exists() {
            fs::copy(&current_exe, &backup_path)?;
            tracing::info!("Created backup at {:?}", backup_path);
        }

        // Set executable permissions on update
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(update_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(update_path, perms)?;
        }

        // Replace current binary with update
        fs::rename(update_path, &current_exe)?;
        tracing::info!("Applied update successfully");

        Ok(())
    }

    /// Rollback to previous version
    pub fn rollback(&self) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("bak");

        if !backup_path.exists() {
            return Err(anyhow!("No backup found for rollback"));
        }

        fs::rename(&backup_path, &current_exe)?;
        tracing::info!("Rolled back to previous version");

        Ok(())
    }

    /// Verify checksum of downloaded file
    pub fn verify_checksum(&self, file_path: &Path, expected_checksum: &str) -> Result<bool> {
        use sha2::{Sha256, Digest};

        let data = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        let checksum = format!("{:x}", result);

        Ok(checksum == expected_checksum)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub auto_check: bool,
    pub auto_download: bool,
    pub auto_install: bool,
    pub check_interval_hours: u32,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            auto_download: false,
            auto_install: false,
            check_interval_hours: 24,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let checker = UpdateChecker::new("test/repo".to_string(), "0.1.0".to_string());

        assert!(checker.is_newer_version("0.1.1").unwrap());
        assert!(checker.is_newer_version("0.2.0").unwrap());
        assert!(checker.is_newer_version("1.0.0").unwrap());
        assert!(!checker.is_newer_version("0.1.0").unwrap());
        assert!(!checker.is_newer_version("0.0.9").unwrap());
    }

    #[test]
    fn test_parse_version() {
        let checker = UpdateChecker::default();

        assert_eq!(checker.parse_version("1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(checker.parse_version("0.0.1").unwrap(), (0, 0, 1));
        assert!(checker.parse_version("invalid").is_err());
    }
}
