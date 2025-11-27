pub mod contacts;
pub mod email;
pub mod google_contacts;
pub mod html;
pub mod json;
pub mod smart;
pub mod sms;
pub mod social;
pub mod streaming_android;
pub mod text;
pub mod xml;

pub use contacts::{AppleContactsConnector, GenericCsvConnector, GoogleContactsConnector};
pub use email::EmailConnector;
pub use google_contacts::GoogleContactsImporter;
pub use html::HtmlImporter;
pub use json::JsonImporter;
pub use smart::SmartImporter;
pub use sms::SmsConnector;
pub use social::{FacebookConnector, InstagramConnector, LinkedInConnector, TwitterConnector};
pub use streaming_android::StreamingAndroidParser;
pub use text::TextImporter;
pub use xml::XmlImporter;

use crate::connector::ConnectorRegistry;
use ai_middleware::SegmindClient;

/// Create a registry with all available connectors pre-registered
pub fn create_default_registry() -> ConnectorRegistry {
    let mut registry = ConnectorRegistry::new();

    // Contact connectors - order matters (more specific first)
    registry.register(Box::new(GoogleContactsConnector::new()));
    registry.register(Box::new(AppleContactsConnector::new()));

    // Format-specific parsers (JSON, XML, HTML, Text)
    registry.register(Box::new(JsonImporter::new()));
    registry.register(Box::new(XmlImporter::new()));
    registry.register(Box::new(HtmlImporter::new()));
    registry.register(Box::new(TextImporter::new()));

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

/// Create a registry with SmartImporter enabled
/// The SmartImporter uses Segmind AI to detect formats and suggest mappings
pub fn create_smart_registry(ai_client: SegmindClient) -> ConnectorRegistry {
    let mut registry = ConnectorRegistry::new();

    // AI-powered smart importer (FIRST - can handle anything)
    registry.register(Box::new(SmartImporter::new(ai_client)));

    // Contact connectors - order matters (more specific first)
    registry.register(Box::new(GoogleContactsConnector::new()));
    registry.register(Box::new(AppleContactsConnector::new()));

    // Format-specific parsers (JSON, XML, HTML, Text)
    registry.register(Box::new(JsonImporter::new()));
    registry.register(Box::new(XmlImporter::new()));
    registry.register(Box::new(HtmlImporter::new()));
    registry.register(Box::new(TextImporter::new()));

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
