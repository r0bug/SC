use std::env;
use tracing::info;

/// Configuration for communication adapters
#[derive(Debug, Clone)]
pub struct CommunicationConfig {
    /// Enable real SMS sending via Twilio
    pub enable_real_sms: bool,
    /// Enable real email sending via SMTP
    pub enable_real_email: bool,
}

impl CommunicationConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let enable_real_sms = env::var("ENABLE_REAL_SMS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let enable_real_email = env::var("ENABLE_REAL_EMAIL")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Self {
            enable_real_sms,
            enable_real_email,
        }
    }

    /// Check if real SMS is enabled and properly configured
    pub fn is_real_sms_available(&self) -> bool {
        if !self.enable_real_sms {
            return false;
        }

        // Check if required Twilio environment variables are set
        env::var("TWILIO_ACCOUNT_SID").is_ok()
            && env::var("TWILIO_AUTH_TOKEN").is_ok()
            && env::var("TWILIO_PHONE_NUMBER").is_ok()
    }

    /// Check if real email is enabled and properly configured
    pub fn is_real_email_available(&self) -> bool {
        if !self.enable_real_email {
            return false;
        }

        // Check if required SMTP environment variables are set
        env::var("SMTP_SERVER").is_ok()
            && env::var("SMTP_USERNAME").is_ok()
            && env::var("SMTP_PASSWORD").is_ok()
            && env::var("SMTP_FROM_ADDRESS").is_ok()
    }

    /// Log the current configuration status
    pub fn log_status(&self) {
        info!("📧 Communication Configuration:");

        if self.is_real_email_available() {
            info!("  ✅ Real email (SMTP): ENABLED");
        } else if self.enable_real_email {
            info!("  ⚠️  Real email: ENABLED but missing credentials");
        } else {
            info!("  🔵 Email: Using MOCK adapter");
        }

        if self.is_real_sms_available() {
            info!("  ✅ Real SMS (Twilio): ENABLED");
        } else if self.enable_real_sms {
            info!("  ⚠️  Real SMS: ENABLED but missing credentials");
        } else {
            info!("  🔵 SMS: Using MOCK adapter");
        }
    }
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_mock() {
        // Without env vars, should default to mock
        let config = CommunicationConfig {
            enable_real_sms: false,
            enable_real_email: false,
        };

        assert!(!config.is_real_sms_available());
        assert!(!config.is_real_email_available());
    }

    #[test]
    fn test_enabled_but_no_credentials() {
        // If enabled but no credentials, should return false
        let config = CommunicationConfig {
            enable_real_sms: true,
            enable_real_email: true,
        };

        // Without env vars set, these should return false
        // (actual credentials checking happens in is_real_*_available methods)
        assert!(!config.is_real_sms_available());
        assert!(!config.is_real_email_available());
    }
}
