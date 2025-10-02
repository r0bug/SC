pub mod segmind;
pub mod suggestions;
pub mod logging;

pub use segmind::SegmindClient;
pub use suggestions::SuggestionEngine;
pub use logging::{LoggingSegmindClient, AiInteractionRecord};