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

/// Get the default owner user ID for Twilio webhook-created contacts.
/// This can be configured via the TWILIO_DEFAULT_OWNER_ID environment variable.
fn get_webhook_owner_id() -> Uuid {
    std::env::var("TWILIO_DEFAULT_OWNER_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(|| {
            warn!("TWILIO_DEFAULT_OWNER_ID not set - using system user for webhook-created contacts");
            // Use a deterministic "system" UUID instead of nil
            Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap()
        })
}

/// Twilio webhook payload for incoming SMS
/// See: https://www.twilio.com/docs/sms/twiml#twilios-request-to-your-application
#[allow(dead_code)]
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
    info!(
        "📱 Received inbound SMS from {} to {}",
        payload.from, payload.to
    );
    info!("📄 Message: {}", payload.body);

    let contact_repo = ContactRepository::new(state.pool.as_ref());
    let sms_repo = SmsHistoryRepository::new(state.pool.as_ref());

    // Get the configurable webhook owner ID
    let webhook_owner_id = get_webhook_owner_id();

    // First, try to find an existing contact by phone number (check across all users)
    let contact_id = match contact_repo.get_by_phone(&payload.from).await {
        Ok(Some(existing_contact)) => {
            // Found existing contact - use it
            info!(
                "✅ Found existing contact: {} {}",
                existing_contact.first_name,
                existing_contact.last_name.as_deref().unwrap_or("")
            );
            Some(existing_contact.id)
        }
        Ok(None) => {
            // No contact found - create one automatically
            info!("📇 Creating new contact for phone: {}", payload.from);

            let new_contact = core_domain::Contact {
                id: Uuid::new_v4(),
                first_name: format!(
                    "SMS Contact {}",
                    &payload.from[..std::cmp::min(10, payload.from.len())]
                ),
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
                created_by: webhook_owner_id,
                version: 1,
                last_synced_at: None,
                metadata: serde_json::json!({"source": "twilio_inbound"}),
            };

            match contact_repo.create(&new_contact).await {
                Ok(_) => {
                    // Create ACL for the new contact
                    let _ = state.acl_service.create_acl(&webhook_owner_id, core_domain::ShareEntityType::Contact, &new_contact.id).await;
                    Some(new_contact.id)
                }
                Err(e) => {
                    warn!("⚠️ Failed to create contact: {}", e);
                    None
                }
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

    sms_repo.create(&sms_message).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to store SMS: {}", e),
        )
    })?;

    info!("✅ SMS stored successfully with ID: {}", sms_message.id);

    // Broadcast via WebSocket for real-time updates
    state
        .ws_broadcaster
        .broadcast(crate::websocket::BroadcastEvent::SmsReceived {
            id: sms_message.id,
            contact_id,
            from: payload.from.clone(),
            body: payload.body.clone(),
        })
        .await;

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
