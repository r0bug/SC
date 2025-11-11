use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use local_store::{ContactRepository, SmsHistoryRepository};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Twilio webhook payload for incoming SMS
/// See: https://www.twilio.com/docs/sms/twiml#twilios-request-to-your-application
#[derive(Debug, Deserialize)]
pub struct TwilioInboundSms {
    #[serde(rename = "MessageSid")]
    message_sid: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Body")]
    body: String,
    #[serde(rename = "NumMedia")]
    num_media: Option<String>,
    #[serde(rename = "FromCity")]
    from_city: Option<String>,
    #[serde(rename = "FromState")]
    from_state: Option<String>,
    #[serde(rename = "FromCountry")]
    from_country: Option<String>,
}

/// POST /api/webhooks/twilio/sms
/// Twilio webhook endpoint for receiving inbound SMS
///
/// This endpoint:
/// 1. Receives SMS from Twilio
/// 2. Finds or creates a contact based on phone number
/// 3. Stores the message in sms_history
/// 4. Broadcasts via WebSocket for real-time updates
///
/// Configure this URL in your Twilio console:
/// https://yourdomain.com/api/webhooks/twilio/sms
pub async fn receive_sms(
    State(state): State<AppState>,
    Form(payload): Form<TwilioInboundSms>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("📱 Received inbound SMS from {} to {}", payload.from, payload.to);
    info!("📄 Message: {}", payload.body);

    let contact_repo = ContactRepository::new(state.pool.as_ref());
    let sms_repo = SmsHistoryRepository::new(state.pool.as_ref());

    // Try to find contact by phone number
    let placeholder_user = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    let contact_id = match contact_repo.search(&payload.from, placeholder_user).await {
        Ok(contacts) => {
            if contacts.is_empty() {
                // No contact found - create one automatically
                info!("📇 Creating new contact for phone: {}", payload.from);

                let new_contact = core_domain::Contact {
                    id: Uuid::new_v4(),
                    first_name: format!("SMS Contact {}", &payload.from[..std::cmp::min(10, payload.from.len())]),
                    last_name: None,
                    email: None,
                    phone: Some(payload.from.clone()),
                    organization: None,
                    title: None,
                    notes: Some("Auto-created from inbound SMS".to_string()),
                    social_handles: vec![],
                    tags: vec![],
                    projects: vec![],
                    groups: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: placeholder_user,
                    version: 1,
                    last_synced_at: None,
                    metadata: serde_json::json!({"source": "twilio_inbound"}),
                };

                contact_repo.create(&new_contact).await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create contact: {}", e)))?;

                Some(new_contact.id)
            } else {
                // Contact found
                let contact = &contacts[0];
                info!("✅ Found existing contact: {} {}", contact.first_name, contact.last_name.as_deref().unwrap_or(""));
                Some(contact.id)
            }
        }
        Err(e) => {
            warn!("⚠️ Error searching for contact: {}", e);
            None
        }
    };

    // Store the inbound SMS in sms_history
    let sms_message = local_store::SmsMessage {
        id: Uuid::new_v4(),
        contact_id,
        phone_number: payload.from.clone(),
        contact_name: None, // Will be populated from contact if linked
        message_date: Utc::now().timestamp_millis(),
        message_type: 1, // 1 = received
        subject: None,
        body: payload.body.clone(),
        readable_date: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        thread_id: None,
        read_status: 0, // 0 = unread
        subscription_id: Some(payload.message_sid.clone()),
        imported_at: Utc::now(),
        imported_by: "twilio_webhook".to_string(),
        source_file: Some(format!("twilio:{}", payload.to)),
    };

    sms_repo.create(&sms_message).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store SMS: {}", e)))?;

    info!("✅ SMS stored successfully with ID: {}", sms_message.id);

    // TODO: Broadcast via WebSocket for real-time updates
    // state.ws_broadcaster.broadcast(...)

    // Respond to Twilio with TwiML (empty response = 200 OK, no reply)
    Ok((
        StatusCode::OK,
        [("Content-Type", "text/xml")],
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Response></Response>".to_string(),
    ))
}

/// GET /api/webhooks/twilio/sms/test
/// Test endpoint to verify webhook is reachable
pub async fn test_webhook() -> impl IntoResponse {
    (
        StatusCode::OK,
        "Twilio webhook endpoint is active! Configure this URL in your Twilio console.",
    )
}
