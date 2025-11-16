use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose, Engine};
use once_cell::sync::OnceCell;
use pbkdf2::pbkdf2_hmac;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{collections::HashMap, env, fs, path::Path};
use thiserror::Error;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 600_000;

static GLOBAL_VAULT: OnceCell<SecureVault> = OnceCell::new();

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("vault file not configured")]
    NotConfigured,
    #[error("missing master key env var")]
    MissingKey,
    #[error("{0}")]
    Io(String),
    #[error("encryption error: {0}")]
    Crypto(String),
    #[error("invalid vault format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    version: u8,
    salt: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Debug, Clone)]
pub struct SecureVault {
    secrets: HashMap<String, String>,
}

impl SecureVault {
    pub fn load_from_file(path: impl AsRef<Path>, master_key: &str) -> Result<Self, VaultError> {
        let raw =
            fs::read_to_string(path.as_ref()).map_err(|e| VaultError::Io(format!("{}", e)))?;
        let vault: VaultFile =
            serde_json::from_str(&raw).map_err(|e| VaultError::InvalidFormat(format!("{}", e)))?;

        if vault.version != 1 {
            return Err(VaultError::InvalidFormat("unsupported version".into()));
        }

        let salt = general_purpose::STANDARD
            .decode(vault.salt)
            .map_err(|e| VaultError::InvalidFormat(format!("invalid salt: {}", e)))?;
        let nonce_bytes = general_purpose::STANDARD
            .decode(vault.nonce)
            .map_err(|e| VaultError::InvalidFormat(format!("invalid nonce: {}", e)))?;
        let ciphertext = general_purpose::STANDARD
            .decode(vault.ciphertext)
            .map_err(|e| VaultError::InvalidFormat(format!("invalid ciphertext: {}", e)))?;

        let key = derive_key(master_key, &salt)?;
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| VaultError::Crypto(format!("decrypt failed: {}", e)))?;

        let env_map =
            parse_env_file(std::str::from_utf8(&plaintext).map_err(|e| {
                VaultError::InvalidFormat(format!("invalid utf8 plaintext: {}", e))
            })?);

        Ok(Self { secrets: env_map })
    }

    pub fn export_to_env(&self) {
        for (key, value) in &self.secrets {
            if env::var_os(key).is_none() {
                env::set_var(key, value);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.secrets.get(key).map(|s| s.as_str())
    }
}

pub fn load_from_env_and_export() -> Result<Option<&'static SecureVault>, VaultError> {
    if let Some(existing) = GLOBAL_VAULT.get() {
        return Ok(Some(existing));
    }

    let path = match env::var("SAGENSCONTACT_VAULT_FILE") {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let master_key = env::var("SAGENSCONTACT_VAULT_KEY").map_err(|_| VaultError::MissingKey)?;

    let vault = SecureVault::load_from_file(path, &master_key)?;
    vault.export_to_env();
    Ok(Some(GLOBAL_VAULT.get_or_init(|| vault)))
}

pub fn encrypt_env_file(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    master_key: &str,
) -> Result<(), VaultError> {
    let plaintext = fs::read_to_string(&input).map_err(|e| VaultError::Io(format!("{}", e)))?;
    let salt = random_bytes(SALT_LEN);
    let nonce = random_bytes(NONCE_LEN);
    let key = derive_key(master_key, &salt)?;
    let cipher = Aes256Gcm::new(&key.into());
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|e| VaultError::Crypto(format!("encrypt failed: {}", e)))?;

    let vault = VaultFile {
        version: 1,
        salt: general_purpose::STANDARD.encode(salt),
        nonce: general_purpose::STANDARD.encode(nonce),
        ciphertext: general_purpose::STANDARD.encode(ciphertext),
    };

    let serialized = serde_json::to_string_pretty(&vault)
        .map_err(|e| VaultError::InvalidFormat(format!("{}", e)))?;
    fs::write(output, serialized).map_err(|e| VaultError::Io(format!("{}", e)))?;
    Ok(())
}

pub fn decrypt_to_string(path: impl AsRef<Path>, master_key: &str) -> Result<String, VaultError> {
    let vault = SecureVault::load_from_file(path, master_key)?;
    let mut buffer = String::new();
    for (key, value) in vault.secrets.iter() {
        buffer.push_str(key);
        buffer.push('=');
        buffer.push_str(value);
        buffer.push('\n');
    }
    Ok(buffer)
}

fn derive_key(master_key: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], VaultError> {
    if salt.len() != SALT_LEN {
        return Err(VaultError::InvalidFormat("salt length mismatch".into()));
    }
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(master_key.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    Ok(key)
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    OsRng.fill_bytes(&mut buf);
    buf
}

fn parse_env_file(contents: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn encrypt_and_decrypt_round_trip() {
        let dir = tempdir().unwrap();
        let plaintext_path = dir.path().join("secrets.env");
        let mut file = File::create(&plaintext_path).unwrap();
        writeln!(file, "DATABASE_URL=sqlite:/tmp/test.db").unwrap();
        writeln!(file, "JWT_SECRET=supersecret").unwrap();

        let vault_path = dir.path().join("secrets.vault");
        encrypt_env_file(&plaintext_path, &vault_path, "master-password").unwrap();

        let vault = SecureVault::load_from_file(&vault_path, "master-password").unwrap();
        assert_eq!(vault.get("DATABASE_URL"), Some("sqlite:/tmp/test.db"));
        assert_eq!(vault.get("JWT_SECRET"), Some("supersecret"));
    }
}
