pub mod sms;
pub mod email;
pub mod contacts;
pub mod social;

pub use sms::SmsConnector;
pub use email::EmailConnector;
pub use contacts::{GoogleContactsConnector, AppleContactsConnector, GenericCsvConnector};
pub use social::{LinkedInConnector, TwitterConnector, FacebookConnector, InstagramConnector};

use crate::connector::ConnectorRegistry;

/// Create a registry with all available connectors pre-registered
pub fn create_default_registry() -> ConnectorRegistry {
    let mut registry = ConnectorRegistry::new();

    // Contact connectors - order matters (more specific first)
    registry.register(Box::new(GoogleContactsConnector::new()));
    registry.register(Box::new(AppleContactsConnector::new()));

    // Email connectors
    registry.register(Box::new(EmailConnector::new()));

    // SMS connectors
    registry.register(Box::new(SmsConnector::new()));

    // Social network connectors
    registry.register(Box::new(LinkedInConnector::new()));
    registry.register(Box::new(TwitterConnector::new()));
    registry.register(Box::new(FacebookConnector::new()));
    registry.register(Box::new(InstagramConnector::new()));

    // Generic fallback (last)
    registry.register(Box::new(GenericCsvConnector::new()));

    registry
}
