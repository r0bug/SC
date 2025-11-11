# Twilio SMS Integration Setup Guide

SagensContact now supports **bidirectional SMS communication** via Twilio! This allows you to send and receive SMS messages through your contacts.

## Features

- ✅ **Outbound SMS**: Send SMS messages to contacts via communication queue
- ✅ **Inbound SMS**: Receive SMS via Twilio webhooks
- ✅ **Auto-contact Creation**: Automatically creates contacts from unknown numbers
- ✅ **SMS History**: All messages stored in `sms_history` table
- ✅ **Mock Mode**: Test without Twilio credentials (default)
- ✅ **Rate Limiting**: 10 req/min, 100 req/hr, 500 req/day

## Prerequisites

1. **Twilio Account**: Sign up at https://www.twilio.com/try-twilio
2. **Phone Number**: Purchase a Twilio phone number with SMS capabilities
3. **Credentials**: Get your Account SID and Auth Token from Twilio Console

## Step 1: Get Twilio Credentials

1. Go to https://console.twilio.com
2. From your Dashboard, copy:
   - **Account SID** (starts with `AC...`)
   - **Auth Token** (click "show" to reveal)
3. Go to **Phone Numbers** → **Manage** → **Active Numbers**
4. Click on your phone number and copy it (format: `+1234567890`)

## Step 2: Configure Environment Variables

### Option A: Environment Variables (Recommended for Production)

```bash
export TWILIO_ACCOUNT_SID="ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
export TWILIO_AUTH_TOKEN="your_auth_token_here"
export TWILIO_PHONE_NUMBER="+1234567890"
```

Add to your `.bashrc` or `.zshrc` for persistence, or use a `.env` file:

```bash
# .env
TWILIO_ACCOUNT_SID=ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_AUTH_TOKEN=your_auth_token_here
TWILIO_PHONE_NUMBER=+1234567890
```

### Option B: Configuration File (Less Secure - Alpha Only)

Create `config/credentials.toml` from the example:

```bash
cp config/credentials.toml.example config/credentials.toml
```

Edit `config/credentials.toml`:

```toml
[sms]
provider = "twilio"  # Change from "mock" to "twilio"
twilio_account_sid = "ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
twilio_auth_token = "your_auth_token_here"
twilio_phone_number = "+1234567890"
```

⚠️ **Security Warning**: Never commit `credentials.toml` to version control!

## Step 3: Configure Inbound SMS Webhook

To receive incoming SMS messages, configure Twilio to send webhooks to your server.

### 3a. Expose Your Server (Development)

If running locally, use ngrok to expose your server:

```bash
# Install ngrok: https://ngrok.com/download
ngrok http 3000
```

You'll get a URL like: `https://abc123.ngrok.io`

### 3b. Configure Twilio Webhook

1. Go to https://console.twilio.com/us1/develop/phone-numbers/manage/incoming
2. Click on your phone number
3. Scroll to **Messaging Configuration**
4. Under **"A MESSAGE COMES IN"**:
   - Select **Webhook**
   - Enter your webhook URL: `https://yourdomain.com/api/webhooks/twilio/sms`
   - HTTP Method: **POST**
5. Click **Save Configuration**

Example webhook URLs:
- Development: `https://abc123.ngrok.io/api/webhooks/twilio/sms`
- Production: `https://api.sagenscontact.com/api/webhooks/twilio/sms`

## Step 4: Start Services

```bash
# Start sync service (port 3000)
DATABASE_URL="sqlite:data/contacts.db" \
JWT_SECRET="your-secret-key-min-32-chars" \
TWILIO_ACCOUNT_SID="ACxxxxxxxx..." \
TWILIO_AUTH_TOKEN="your_token" \
TWILIO_PHONE_NUMBER="+1234567890" \
cargo run --release --bin sync_service

# Start worker (processes outbound SMS)
cargo run --release --bin worker
```

You should see:
```
✅ Using Twilio SMS adapter (real SMS sending enabled)
```

If you see:
```
ℹ️ Using Mock SMS adapter (set TWILIO_* env vars for real SMS)
```

This means Twilio credentials were not found and mock mode is active.

## Step 5: Test the Integration

### Test Outbound SMS

```bash
# Via CLI
./target/release/sagenscontact communicate --contact-id <uuid> \
  --method sms \
  --content "Hello from SagensContact!"

# Via API
curl -X POST http://localhost:3000/api/communications \
  -H "Content-Type: application/json" \
  -d '{
    "contact_id": "your-contact-uuid",
    "method": "sms",
    "content": "Hello from SagensContact!",
    "scheduled_at": null
  }'
```

### Test Inbound SMS

1. **Test webhook endpoint**:
   ```bash
   curl http://localhost:3000/api/webhooks/twilio/sms/test
   ```

   You should see: `Twilio webhook endpoint is active!`

2. **Send SMS to your Twilio number** from your phone

3. **Check the logs** - you should see:
   ```
   📱 Received inbound SMS from +1234567890 to +0987654321
   📄 Message: Hello!
   ```

4. **Verify in database**:
   ```bash
   sqlite3 data/contacts.db "SELECT * FROM sms_history ORDER BY imported_at DESC LIMIT 5;"
   ```

## Architecture

### Outbound SMS Flow

```
User/App → API → CommunicationQueue → TwilioSmsAdapter → Twilio API → Recipient
                       ↓
                  sms_history table
```

### Inbound SMS Flow

```
Sender → Twilio → Webhook (/api/webhooks/twilio/sms) → SagensContact
                                ↓
                    1. Find/Create Contact
                    2. Store in sms_history
                    3. Broadcast via WebSocket (TODO)
```

## Database Schema

All SMS messages (inbound and outbound) are stored in `sms_history`:

```sql
CREATE TABLE sms_history (
    id TEXT PRIMARY KEY,
    contact_id TEXT,  -- Links to contacts table
    phone_number TEXT NOT NULL,
    contact_name TEXT,
    message_date INTEGER NOT NULL,  -- Unix timestamp (ms)
    message_type INTEGER NOT NULL,  -- 1=received, 2=sent
    subject TEXT,
    body TEXT NOT NULL,
    readable_date TEXT NOT NULL,
    thread_id INTEGER,
    read_status INTEGER DEFAULT 0,  -- 0=unread, 1=read
    subscription_id TEXT,           -- Twilio MessageSid
    imported_at TEXT NOT NULL,
    imported_by TEXT NOT NULL,      -- "twilio_webhook" or "communication_queue"
    source_file TEXT
);
```

## Security Considerations

### Production Deployment

1. **HTTPS Only**: Twilio webhooks require HTTPS in production
2. **Webhook Validation**: Implement Twilio signature validation (TODO)
3. **Rate Limiting**: Already implemented (10/min, 100/hr, 500/day)
4. **Credential Storage**: Use environment variables or secret manager
5. **Database Encryption**: Enable SQLCipher for encrypted storage

### Webhook Security (TODO)

Add Twilio signature validation:

```rust
use twilio_signature::validate;

fn validate_twilio_signature(headers: &HeaderMap, body: &str, url: &str) -> bool {
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN").unwrap();
    let signature = headers.get("X-Twilio-Signature").unwrap().to_str().unwrap();
    validate(&auth_token, &signature, url, body)
}
```

## Troubleshooting

### "Using Mock SMS adapter" message

**Problem**: Twilio credentials not found

**Solution**: Verify environment variables are set:
```bash
env | grep TWILIO
```

### SMS not sending

**Problem**: Worker not processing queue

**Solution**: Check worker logs:
```bash
cargo run --bin worker
```

Look for: `Processing batch of X communications`

### Webhook not receiving messages

**Problem**: Twilio can't reach your server

**Solutions**:
1. Verify ngrok is running: `curl https://your-ngrok-url.ngrok.io/api/webhooks/twilio/sms/test`
2. Check Twilio webhook configuration
3. View Twilio debugger: https://console.twilio.com/us1/monitor/logs/debugger

### "Failed to create contact" error

**Problem**: Database permissions or invalid user_id

**Solution**: Webhook uses placeholder user `00000000-0000-0000-0000-000000000000`.
For multi-tenant, map phone numbers to user accounts.

## API Reference

### Webhook Endpoint

**POST** `/api/webhooks/twilio/sms`

Receives Twilio webhook payload:

```
MessageSid=SMxxxxxxxxx
From=+1234567890
To=+0987654321
Body=Hello!
NumMedia=0
```

Response: TwiML (XML)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Response></Response>
```

### Test Endpoint

**GET** `/api/webhooks/twilio/sms/test`

Returns: `200 OK` with status message

## Cost Estimates

- **Twilio Pricing** (US, as of 2024):
  - Phone Number: ~$1.15/month
  - Outbound SMS: $0.0079/message
  - Inbound SMS: $0.0079/message

Example: 1000 SMS/month = ~$16/month

## Next Steps

- [ ] Implement Twilio signature validation for webhook security
- [ ] Add MMS support for image attachments
- [ ] Implement SMS threading/conversations
- [ ] Add WebSocket broadcast for real-time inbound SMS notifications
- [ ] Support multiple Twilio phone numbers
- [ ] Add SMS templates and bulk sending

## Support

- Twilio Docs: https://www.twilio.com/docs/sms
- SagensContact Issues: https://github.com/yourusername/sagenscontact/issues

---

**Happy texting! 📱**
