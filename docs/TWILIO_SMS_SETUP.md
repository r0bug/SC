# Twilio SMS Setup Guide

This guide explains how to configure SagensContact to send real SMS messages via Twilio.

## Overview

SagensContact supports two modes for SMS sending:
- **Mock Mode** (default): SMS sending is simulated and logged. No actual messages are sent.
- **Real Mode**: SMS messages are sent via Twilio's REST API.

Similarly, email can be configured for:
- **Mock Mode** (default): Email sending is simulated.
- **Real Mode**: Emails are sent via SMTP.

## Prerequisites

- A Twilio account (free trial available at https://www.twilio.com/try-twilio)
- A verified phone number or Twilio phone number
- SagensContact sync service and worker running

## Step 1: Create Twilio Account

1. Visit https://www.twilio.com/try-twilio
2. Sign up for a free trial account
3. Verify your email address
4. Verify your phone number (for trial accounts)

## Step 2: Get Twilio Credentials

Once logged into the Twilio console:

1. Navigate to your **Dashboard** (https://console.twilio.com/)
2. Find your **Account SID** and **Auth Token**:
   - Account SID: starts with `AC...`
   - Auth Token: Click "Show" to reveal it (keep this secret!)
3. Get a Twilio phone number:
   - For **trial accounts**: Use the trial number provided
   - For **paid accounts**: Navigate to "Phone Numbers" → "Buy a Number"
   - Choose a number with SMS capabilities
   - Note the phone number in E.164 format (e.g., `+15555550100`)

## Step 3: Configure SagensContact

### Option A: Using Environment Variables (Development)

Create or edit `.env` file in your SagensContact root directory:

```bash
# Enable real SMS sending
ENABLE_REAL_SMS=true

# Twilio credentials
TWILIO_ACCOUNT_SID=ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_AUTH_TOKEN=your_auth_token_here
TWILIO_PHONE_NUMBER=+15555550100
```

### Option B: Using Secure Vault (Production)

1. Edit `config/credentials.env`:

```bash
ENABLE_REAL_SMS=true
TWILIO_ACCOUNT_SID=ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_AUTH_TOKEN=your_auth_token_here
TWILIO_PHONE_NUMBER=+15555550100
```

2. Encrypt the credentials using the vault tool:

```bash
cargo run --bin vault encrypt
```

3. The encrypted credentials will be stored in `config/credentials.vault`

## Step 4: Enable Real Email (Optional)

To also enable real email sending via SMTP, add these to your `.env` or `config/credentials.env`:

```bash
ENABLE_REAL_EMAIL=true
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-specific-password
SMTP_FROM_ADDRESS=your-email@gmail.com
SMTP_FROM_NAME=SagensContact
```

**Note for Gmail users**: You'll need to generate an app-specific password:
1. Go to Google Account settings
2. Security → 2-Step Verification → App passwords
3. Generate a new app password for "Mail"

## Step 5: Restart Services

Restart the SagensContact worker and sync service to load the new configuration:

```bash
# If using the start script
./stop.sh
./start.sh

# Or manually
cargo run --release --bin worker
cargo run --release --bin sync_service
```

When the worker starts, you should see log messages indicating the adapter mode:

```
📧 Communication Configuration:
  ✅ Real SMS (Twilio): ENABLED
  🔵 Email: Using MOCK adapter
```

## Step 6: Test SMS Sending

### Method 1: Via API

Use the test endpoint to verify your configuration:

```bash
curl -X POST http://localhost:3000/api/communications/test \
  -H "Content-Type: application/json" \
  -d '{
    "communication_type": "sms",
    "to": "+15555550199",
    "body": "Test message from SagensContact"
  }'
```

**Expected response (success):**
```json
{
  "success": true,
  "message": "SMS sent successfully (real)",
  "mode": "real"
}
```

### Method 2: Via CLI

```bash
# Create a communication attempt
cargo run --bin sagenscontact send-sms \
  --contact-id <CONTACT_UUID> \
  --message "Test message"
```

The worker will pick up pending communications every 30 seconds.

### Method 3: Check Configuration Status

```bash
curl http://localhost:3000/api/communications/config
```

**Expected response:**
```json
{
  "sms_mode": "real",
  "email_mode": "mock",
  "sms_enabled": true,
  "email_enabled": false,
  "sms_configured": true,
  "email_configured": false
}
```

## Troubleshooting

### "Failed to create queue with config" Error

**Cause**: Missing or invalid Twilio credentials.

**Solution**:
1. Verify all three environment variables are set:
   - `TWILIO_ACCOUNT_SID`
   - `TWILIO_AUTH_TOKEN`
   - `TWILIO_PHONE_NUMBER`
2. Restart the worker after setting the variables

### "Twilio API error: 401 - Authenticate" Error

**Cause**: Invalid Account SID or Auth Token.

**Solution**:
1. Double-check your Account SID and Auth Token in the Twilio console
2. Ensure there are no extra spaces or quotes in the `.env` file
3. For Auth Token, click "Show" in the Twilio console to reveal the full value

### "Invalid phone number" Error

**Cause**: Phone number not in E.164 format.

**Solution**:
- Use E.164 format: `+[country code][number]`
- Examples:
  - US: `+15555550100`
  - UK: `+447700900000`
  - Australia: `+61491570000`

### "Trial account restrictions" Error

**Cause**: Trying to send to an unverified number with a trial account.

**Solution**:
- **Trial accounts** can only send to verified phone numbers
- Verify recipient numbers at: https://console.twilio.com/us1/develop/phone-numbers/manage/verified
- Or upgrade to a paid account for unrestricted sending

### Messages not sending

**Possible causes**:
1. Worker not running: `cargo run --bin worker`
2. Configuration not loaded: Restart the worker
3. Communication attempts stuck in queue: Check database `communication_attempts` table
4. Rate limiting: Twilio trial accounts have sending limits

**Check logs**:
```bash
RUST_LOG=debug cargo run --bin worker
```

Look for:
```
[TWILIO] Sending SMS to +15555550199 from +15555550100
[TWILIO] SMS sent successfully to +15555550199
```

## Cost and Limits

### Twilio Trial Account
- **Credit**: $15 trial credit
- **SMS Cost**: ~$0.0075 per SMS (US/Canada)
- **Restrictions**: Can only send to verified numbers
- **Rate Limits**: 1 SMS per second

### Twilio Paid Account
- **SMS Cost**: Varies by country (~$0.0075-$0.05 per SMS)
- **No sending restrictions**
- **Higher rate limits**
- **Pay-as-you-go pricing**

See https://www.twilio.com/sms/pricing for current pricing.

## Webhook Setup (Receiving SMS)

To receive inbound SMS messages, configure a webhook in your Twilio console:

1. Go to: https://console.twilio.com/us1/develop/phone-numbers/manage/incoming
2. Click on your phone number
3. Under "Messaging Configuration":
   - **A MESSAGE COMES IN**: `https://yourdomain.com/api/webhooks/twilio/sms`
   - **HTTP Method**: `POST`
4. Save the configuration

**Note**: For local development, use a tool like [ngrok](https://ngrok.com/) to expose your local server:

```bash
ngrok http 3000
# Use the ngrok URL in Twilio webhook config
# Example: https://abc123.ngrok.io/api/webhooks/twilio/sms
```

## Security Best Practices

1. **Never commit credentials to version control**
   - Add `.env` to `.gitignore`
   - Use the secure vault for production

2. **Use environment-specific credentials**
   - Development: Use a separate Twilio test account
   - Production: Use a dedicated production account

3. **Rotate credentials regularly**
   - Regenerate Auth Token periodically
   - Update in both Twilio console and SagensContact config

4. **Monitor usage**
   - Set up usage alerts in Twilio console
   - Review monthly costs and usage reports

5. **Use least privilege**
   - Consider creating Twilio API keys with restricted permissions
   - See: https://www.twilio.com/docs/iam/api-keys

## Switching Back to Mock Mode

To disable real SMS sending and revert to mock mode:

1. Edit `.env` or `config/credentials.env`:
   ```bash
   ENABLE_REAL_SMS=false
   ```

2. Restart the worker:
   ```bash
   cargo run --bin worker
   ```

3. Verify in logs:
   ```
   📧 Communication Configuration:
     🔵 SMS: Using MOCK adapter
   ```

## Additional Resources

- [Twilio SMS Quickstart](https://www.twilio.com/docs/sms/quickstart)
- [Twilio SMS API Reference](https://www.twilio.com/docs/sms/api)
- [Twilio Error Codes](https://www.twilio.com/docs/api/errors)
- [Twilio Console](https://console.twilio.com/)
- [SagensContact Documentation](../README.md)

## Support

If you encounter issues not covered in this guide:

1. Check the logs: `RUST_LOG=debug cargo run --bin worker`
2. Review Twilio console for error details
3. Open an issue on GitHub: [SagensContact Issues](https://github.com/yourusername/sagenscontact/issues)
4. Join our community Discord: [link]

---

**Last Updated**: 2024-11-16
**Version**: Alpha v0.1.0
