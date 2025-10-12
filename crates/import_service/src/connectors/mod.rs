pub mod contacts;
pub mod email;
pub mod google_contacts;
pub mod sms;
pub mod social;
pub mod streaming_android;

pub use contacts::{AppleContactsConnector, GenericCsvConnector, GoogleContactsConnector};
pub use email::EmailConnector;
pub use google_contacts::GoogleContactsImporter;
pub use sms::SmsConnector;
pub use social::{FacebookConnector, InstagramConnector, LinkedInConnector, TwitterConnector};
pub use streaming_android::StreamingAndroidParser;

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
