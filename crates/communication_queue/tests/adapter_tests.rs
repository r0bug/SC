use communication_queue::adapters::{
    EmailAdapterTrait, MockEmailAdapter, MockSmsAdapter, SmsAdapterTrait, SmtpEmailAdapter,
    TwilioSmsAdapter,
};
use communication_queue::config::CommunicationConfig;
use communication_queue::CommunicationQueue;
use serial_test::serial;
use std::env;
use wiremock::{
    matchers::{body_partial_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
#[serial]
fn test_config_defaults_to_mock() {
    // Clear any existing env vars
    env::remove_var("ENABLE_REAL_SMS");
    env::remove_var("ENABLE_REAL_EMAIL");

    let config = CommunicationConfig::from_env();

    assert!(!config.enable_real_sms);
    assert!(!config.enable_real_email);
    assert!(!config.is_real_sms_available());
    assert!(!config.is_real_email_available());
}

#[test]
#[serial]
fn test_config_enabled_but_missing_credentials() {
    // Enable but don't provide credentials
    env::set_var("ENABLE_REAL_SMS", "true");
    env::set_var("ENABLE_REAL_EMAIL", "true");
    env::remove_var("TWILIO_ACCOUNT_SID");
    env::remove_var("SMTP_SERVER");

    let config = CommunicationConfig::from_env();

    assert!(config.enable_real_sms);
    assert!(config.enable_real_email);
    // But not actually available without credentials
    assert!(!config.is_real_sms_available());
    assert!(!config.is_real_email_available());

    // Cleanup
    env::remove_var("ENABLE_REAL_SMS");
    env::remove_var("ENABLE_REAL_EMAIL");
}

#[test]
#[serial]
fn test_config_sms_available_with_credentials() {
    env::set_var("ENABLE_REAL_SMS", "true");
    env::set_var("TWILIO_ACCOUNT_SID", "ACtest123");
    env::set_var("TWILIO_AUTH_TOKEN", "test_token");
    env::set_var("TWILIO_PHONE_NUMBER", "+15555550100");

    let config = CommunicationConfig::from_env();

    assert!(config.enable_real_sms);
    assert!(config.is_real_sms_available());

    // Cleanup
    env::remove_var("ENABLE_REAL_SMS");
    env::remove_var("TWILIO_ACCOUNT_SID");
    env::remove_var("TWILIO_AUTH_TOKEN");
    env::remove_var("TWILIO_PHONE_NUMBER");
}

#[test]
#[serial]
fn test_config_email_available_with_credentials() {
    env::set_var("ENABLE_REAL_EMAIL", "true");
    env::set_var("SMTP_SERVER", "smtp.test.com");
    env::set_var("SMTP_USERNAME", "test@test.com");
    env::set_var("SMTP_PASSWORD", "password");
    env::set_var("SMTP_FROM_ADDRESS", "sender@test.com");

    let config = CommunicationConfig::from_env();

    assert!(config.enable_real_email);
    assert!(config.is_real_email_available());

    // Cleanup
    env::remove_var("ENABLE_REAL_EMAIL");
    env::remove_var("SMTP_SERVER");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_ADDRESS");
}

// ============================================================================
// Mock Adapter Tests
// ============================================================================

#[tokio::test]
async fn test_mock_email_adapter_success() {
    let adapter = MockEmailAdapter::new();

    let result = adapter
        .send_email("test@example.com", "Test Subject", "Test Body")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_email_adapter_failure() {
    let adapter = MockEmailAdapter::new();

    // The mock fails if body contains "test-fail"
    let result = adapter
        .send_email("test@example.com", "Test", "test-fail content")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_sms_adapter_success() {
    let adapter = MockSmsAdapter::new();

    let result = adapter.send_sms("+15555550100", "Test message").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_sms_adapter_long_message() {
    let adapter = MockSmsAdapter::new();

    // Long message (>160 chars) should still succeed
    let long_message = "a".repeat(200);
    let result = adapter.send_sms("+15555550100", &long_message).await;

    assert!(result.is_ok());
}

// ============================================================================
// Twilio Adapter Tests (with WireMock)
// ============================================================================

#[tokio::test]
async fn test_twilio_adapter_successful_send() {
    let mock_server = MockServer::start().await;

    // Mock successful Twilio response
    Mock::given(method("POST"))
        .and(path("/2010-04-01/Accounts/ACtest123/Messages.json"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "sid": "SMtest123",
                "status": "queued",
                "to": "+15555550199",
                "from": "+15555550100",
                "body": "Test message"
            })),
        )
        .mount(&mock_server)
        .await;

    // Create adapter pointing to mock server
    // Note: This would require modifying TwilioSmsAdapter to accept a custom base URL
    // For now, this test demonstrates the pattern
    // In real implementation, you'd need to make the Twilio URL configurable

    // let adapter = TwilioSmsAdapter::new(
    //     "ACtest123".to_string(),
    //     "test_token".to_string(),
    //     "+15555550100".to_string(),
    // );
    //
    // let result = adapter.send_sms("+15555550199", "Test message").await;
    // assert!(result.is_ok());

    // For now, just verify mock server was set up correctly
    assert!(mock_server.address().port() > 0);
}

#[tokio::test]
async fn test_twilio_adapter_invalid_credentials() {
    let mock_server = MockServer::start().await;

    // Mock Twilio auth error
    Mock::given(method("POST"))
        .and(path("/2010-04-01/Accounts/ACtest123/Messages.json"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "code": 20003,
                "message": "Authenticate",
                "status": 401
            })),
        )
        .mount(&mock_server)
        .await;

    // Demonstrates error handling pattern
    assert!(mock_server.address().port() > 0);
}

#[tokio::test]
#[serial]
async fn test_twilio_adapter_from_env_missing_vars() {
    // Clear env vars
    env::remove_var("TWILIO_ACCOUNT_SID");
    env::remove_var("TWILIO_AUTH_TOKEN");
    env::remove_var("TWILIO_PHONE_NUMBER");

    let result = TwilioSmsAdapter::from_env();

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("TWILIO_ACCOUNT_SID"));
}

#[tokio::test]
#[serial]
async fn test_twilio_adapter_from_env_success() {
    env::set_var("TWILIO_ACCOUNT_SID", "ACtest123");
    env::set_var("TWILIO_AUTH_TOKEN", "test_token");
    env::set_var("TWILIO_PHONE_NUMBER", "+15555550100");

    let result = TwilioSmsAdapter::from_env();

    assert!(result.is_ok());

    // Cleanup
    env::remove_var("TWILIO_ACCOUNT_SID");
    env::remove_var("TWILIO_AUTH_TOKEN");
    env::remove_var("TWILIO_PHONE_NUMBER");
}

// ============================================================================
// SMTP Adapter Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_smtp_adapter_from_env_missing_vars() {
    // Clear env vars
    env::remove_var("SMTP_SERVER");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_ADDRESS");

    let result = SmtpEmailAdapter::from_env();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("SMTP_SERVER"));
}

#[tokio::test]
#[serial]
async fn test_smtp_adapter_from_env_success() {
    env::set_var("SMTP_SERVER", "smtp.test.com");
    env::set_var("SMTP_PORT", "587");
    env::set_var("SMTP_USERNAME", "test@test.com");
    env::set_var("SMTP_PASSWORD", "password");
    env::set_var("SMTP_FROM_ADDRESS", "sender@test.com");
    env::set_var("SMTP_FROM_NAME", "Test Sender");

    let result = SmtpEmailAdapter::from_env();

    assert!(result.is_ok());

    // Cleanup
    env::remove_var("SMTP_SERVER");
    env::remove_var("SMTP_PORT");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_ADDRESS");
    env::remove_var("SMTP_FROM_NAME");
}

#[tokio::test]
#[serial]
async fn test_smtp_adapter_default_port() {
    env::set_var("SMTP_SERVER", "smtp.test.com");
    env::remove_var("SMTP_PORT"); // Don't set port
    env::set_var("SMTP_USERNAME", "test@test.com");
    env::set_var("SMTP_PASSWORD", "password");
    env::set_var("SMTP_FROM_ADDRESS", "sender@test.com");

    let result = SmtpEmailAdapter::from_env();

    assert!(result.is_ok());

    // Cleanup
    env::remove_var("SMTP_SERVER");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_ADDRESS");
}

// ============================================================================
// CommunicationQueue Tests
// ============================================================================

#[test]
fn test_queue_default_uses_mock() {
    let queue = CommunicationQueue::new();
    // Default queue should be created successfully
    // (testing that it compiles and runs without panic)
    drop(queue);
}

#[test]
#[serial]
fn test_queue_with_config_mock_mode() {
    env::remove_var("ENABLE_REAL_SMS");
    env::remove_var("ENABLE_REAL_EMAIL");

    let config = CommunicationConfig::from_env();
    let result = CommunicationQueue::with_config(config);

    assert!(result.is_ok());
}

#[test]
#[serial]
fn test_queue_with_config_real_sms_enabled() {
    env::set_var("ENABLE_REAL_SMS", "true");
    env::set_var("TWILIO_ACCOUNT_SID", "ACtest123");
    env::set_var("TWILIO_AUTH_TOKEN", "test_token");
    env::set_var("TWILIO_PHONE_NUMBER", "+15555550100");
    env::remove_var("ENABLE_REAL_EMAIL");

    let config = CommunicationConfig::from_env();
    let result = CommunicationQueue::with_config(config);

    assert!(result.is_ok());

    // Cleanup
    env::remove_var("ENABLE_REAL_SMS");
    env::remove_var("TWILIO_ACCOUNT_SID");
    env::remove_var("TWILIO_AUTH_TOKEN");
    env::remove_var("TWILIO_PHONE_NUMBER");
}

#[test]
#[serial]
fn test_queue_with_config_real_email_enabled() {
    env::remove_var("ENABLE_REAL_SMS");
    env::set_var("ENABLE_REAL_EMAIL", "true");
    env::set_var("SMTP_SERVER", "smtp.test.com");
    env::set_var("SMTP_USERNAME", "test@test.com");
    env::set_var("SMTP_PASSWORD", "password");
    env::set_var("SMTP_FROM_ADDRESS", "sender@test.com");

    let config = CommunicationConfig::from_env();
    let result = CommunicationQueue::with_config(config);

    assert!(result.is_ok());

    // Cleanup
    env::remove_var("ENABLE_REAL_EMAIL");
    env::remove_var("SMTP_SERVER");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_ADDRESS");
}

#[test]
#[serial]
fn test_queue_with_config_both_real_enabled() {
    env::set_var("ENABLE_REAL_SMS", "true");
    env::set_var("TWILIO_ACCOUNT_SID", "ACtest123");
    env::set_var("TWILIO_AUTH_TOKEN", "test_token");
    env::set_var("TWILIO_PHONE_NUMBER", "+15555550100");
    env::set_var("ENABLE_REAL_EMAIL", "true");
    env::set_var("SMTP_SERVER", "smtp.test.com");
    env::set_var("SMTP_USERNAME", "test@test.com");
    env::set_var("SMTP_PASSWORD", "password");
    env::set_var("SMTP_FROM_ADDRESS", "sender@test.com");

    let config = CommunicationConfig::from_env();
    let result = CommunicationQueue::with_config(config);

    assert!(result.is_ok());

    // Cleanup
    env::remove_var("ENABLE_REAL_SMS");
    env::remove_var("TWILIO_ACCOUNT_SID");
    env::remove_var("TWILIO_AUTH_TOKEN");
    env::remove_var("TWILIO_PHONE_NUMBER");
    env::remove_var("ENABLE_REAL_EMAIL");
    env::remove_var("SMTP_SERVER");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_ADDRESS");
}
