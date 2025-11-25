use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use communication_queue::CommunicationConfig;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::state::AppState;

/// Response for GET /api/communications/config
#[derive(Debug, Serialize)]
pub struct ConfigStatusResponse {
    pub sms_mode: String,
    pub email_mode: String,
    pub sms_enabled: bool,
    pub email_enabled: bool,
    pub sms_configured: bool,
    pub email_configured: bool,
}

/// GET /api/communications/config
/// Returns the current communication configuration status
pub async fn get_config_status() -> impl IntoResponse {
    let config = CommunicationConfig::from_env();

    let sms_configured = config.is_real_sms_available();
    let email_configured = config.is_real_email_available();

    let response = ConfigStatusResponse {
        sms_mode: if sms_configured {
            "real".to_string()
        } else {
            "mock".to_string()
        },
        email_mode: if email_configured {
            "real".to_string()
        } else {
            "mock".to_string()
        },
        sms_enabled: config.enable_real_sms,
        email_enabled: config.enable_real_email,
        sms_configured,
        email_configured,
    };

    Json(response)
}

/// Request payload for POST /api/communications/test
#[derive(Debug, Deserialize)]
pub struct TestCommunicationRequest {
    pub communication_type: String, // "email" or "sms"
    pub to: String,
    pub subject: Option<String>,
    pub body: String,
}

/// Response for POST /api/communications/test
#[derive(Debug, Serialize)]
pub struct TestCommunicationResponse {
    pub success: bool,
    pub message: String,
    pub mode: String, // "real" or "mock"
}

/// POST /api/communications/test
/// Send a test communication (email or SMS) to verify configuration
pub async fn send_test_communication(
    State(_state): State<AppState>,
    Json(payload): Json<TestCommunicationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(
        "📧 Testing {} communication to: {}",
        payload.communication_type, payload.to
    );

    let config = CommunicationConfig::from_env();

    match payload.communication_type.as_str() {
        "email" => {
            // Create email adapter based on config
            let adapter: Box<dyn communication_queue::adapters::EmailAdapterTrait + Send + Sync> =
                if config.is_real_email_available() {
                    info!("Using real SMTP adapter for test");
                    match communication_queue::adapters::SmtpEmailAdapter::from_env() {
                        Ok(adapter) => Box::new(adapter),
                        Err(e) => {
                            error!("Failed to create SMTP adapter: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to initialize SMTP adapter: {}", e),
                            ));
                        }
                    }
                } else {
                    info!("Using mock email adapter for test");
                    Box::new(communication_queue::adapters::MockEmailAdapter::new())
                };

            let subject = payload.subject.as_deref().unwrap_or("Test Email from SagensContact");

            match adapter.send_email(&payload.to, subject, &payload.body).await {
                Ok(_) => {
                    let mode = if config.is_real_email_available() {
                        "real"
                    } else {
                        "mock"
                    };
                    Ok(Json(TestCommunicationResponse {
                        success: true,
                        message: format!("Email sent successfully ({})", mode),
                        mode: mode.to_string(),
                    }))
                }
                Err(e) => {
                    error!("Failed to send test email: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to send email: {}", e),
                    ))
                }
            }
        }
        "sms" => {
            // Create SMS adapter based on config
            let adapter: Box<dyn communication_queue::adapters::SmsAdapterTrait + Send + Sync> =
                if config.is_real_sms_available() {
                    info!("Using real Twilio adapter for test");
                    match communication_queue::adapters::TwilioSmsAdapter::from_env() {
                        Ok(adapter) => Box::new(adapter),
                        Err(e) => {
                            error!("Failed to create Twilio adapter: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to initialize Twilio adapter: {}", e),
                            ));
                        }
                    }
                } else {
                    info!("Using mock SMS adapter for test");
                    Box::new(communication_queue::adapters::MockSmsAdapter::new())
                };

            match adapter.send_sms(&payload.to, &payload.body).await {
                Ok(_) => {
                    let mode = if config.is_real_sms_available() {
                        "real"
                    } else {
                        "mock"
                    };
                    Ok(Json(TestCommunicationResponse {
                        success: true,
                        message: format!("SMS sent successfully ({})", mode),
                        mode: mode.to_string(),
                    }))
                }
                Err(e) => {
                    error!("Failed to send test SMS: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to send SMS: {}", e),
                    ))
                }
            }
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid communication_type: {}. Use 'email' or 'sms'.",
                payload.communication_type
            ),
        )),
    }
}
