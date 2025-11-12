# Email Integration Setup

SagensContact supports bidirectional email integration with AI-powered importance analysis. This guide explains how to set up SMTP sending and IMAP receiving.

## Features

- **SMTP Email Sending**: Send emails through any SMTP server (Gmail, Outlook, custom server)
- **IMAP Email Receiving**: Automatically poll for new emails and store them in the database
- **Auto-Contact Creation**: Automatically create contacts from unknown email senders
- **AI Importance Analysis**: Score emails 0-10 for importance and detect urgency
- **Full-Text Search**: Search email history by subject, body, sender
- **Email Threading**: Track conversation threads via Message-ID headers

## Environment Variables

### SMTP Configuration (Outgoing Email)

```bash
# SMTP server settings
SMTP_SERVER=smtp.gmail.com          # SMTP server hostname
SMTP_PORT=587                        # SMTP port (587 for TLS, 465 for SSL)
SMTP_USERNAME=your-email@gmail.com   # SMTP username (usually your email)
SMTP_PASSWORD=your-app-password      # SMTP password or app-specific password

# From address settings
SMTP_FROM_ADDRESS=your-email@gmail.com  # Email address for "From" field
SMTP_FROM_NAME="SagensContact"          # Display name for "From" field (optional)
```

### IMAP Configuration (Incoming Email)

```bash
# IMAP server settings
IMAP_SERVER=imap.gmail.com          # IMAP server hostname
IMAP_PORT=993                        # IMAP port (993 for TLS)
IMAP_USERNAME=your-email@gmail.com   # IMAP username (usually your email)
IMAP_PASSWORD=your-app-password      # IMAP password or app-specific password

# Polling settings
IMAP_POLL_INTERVAL=300              # Poll interval in seconds (default: 300 = 5 minutes)
```

### AI Analysis Configuration (Optional)

```bash
# Enable real AI analysis (otherwise uses rule-based analysis)
SEGMIND_API_KEY=your-api-key-here   # Or OPENAI_API_KEY
```

## Provider-Specific Setup

### Gmail

1. **Enable IMAP** in Gmail settings:
   - Go to Settings → Forwarding and POP/IMAP
   - Enable IMAP

2. **Create App Password** (if using 2FA):
   - Go to https://myaccount.google.com/apppasswords
   - Generate an app password for "Mail"
   - Use this password instead of your Google account password

3. **Environment Variables**:
```bash
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

IMAP_SERVER=imap.gmail.com
IMAP_PORT=993
IMAP_USERNAME=your-email@gmail.com
IMAP_PASSWORD=your-app-password

SMTP_FROM_ADDRESS=your-email@gmail.com
SMTP_FROM_NAME="Your Name"
```

### Outlook/Office 365

```bash
SMTP_SERVER=smtp-mail.outlook.com
SMTP_PORT=587
SMTP_USERNAME=your-email@outlook.com
SMTP_PASSWORD=your-password

IMAP_SERVER=outlook.office365.com
IMAP_PORT=993
IMAP_USERNAME=your-email@outlook.com
IMAP_PASSWORD=your-password

SMTP_FROM_ADDRESS=your-email@outlook.com
SMTP_FROM_NAME="Your Name"
```

### Custom SMTP/IMAP Server

```bash
SMTP_SERVER=mail.yourdomain.com
SMTP_PORT=587
SMTP_USERNAME=you@yourdomain.com
SMTP_PASSWORD=your-password

IMAP_SERVER=mail.yourdomain.com
IMAP_PORT=993
IMAP_USERNAME=you@yourdomain.com
IMAP_PASSWORD=your-password

SMTP_FROM_ADDRESS=you@yourdomain.com
SMTP_FROM_NAME="Your Name"
```

## Starting the Worker

Once configured, start the worker to begin email monitoring:

```bash
# From the alpha/ directory
DATABASE_URL="sqlite:data/contacts.db" ./target/release/worker
```

You should see:
```
📧 Email monitor started
ℹ️  Email: Set SMTP_* env vars for real sending, IMAP_* for receiving
```

If IMAP is not configured, you'll see:
```
ℹ️  Email monitor disabled (set IMAP_* env vars to enable)
```

## How It Works

### IMAP Email Receiving

1. **Polling**: Worker polls IMAP server every `IMAP_POLL_INTERVAL` seconds (default 5 minutes)
2. **Fetch Unread**: Retrieves all unread emails from INBOX
3. **Parse Email**: Extracts sender, subject, body (text and HTML), headers
4. **Find/Create Contact**:
   - Searches for existing contact by email address
   - Auto-creates contact if not found (first name from display name or email prefix)
5. **Store in Database**: Saves to `email_history` table with full metadata
6. **AI Analysis**: Analyzes importance (0-10 score), urgency, and generates summary

### SMTP Email Sending

1. **Queue Email**: Create a `CommunicationAttempt` with method `Email`
2. **Worker Processing**: Background worker processes queue
3. **SMTP Send**: Uses configured SMTP server to send email
4. **Update Status**: Marks as `Sent` or `Failed` in database

### AI Importance Analysis

**Rule-Based Algorithm** (used when no AI API key is configured):
- Baseline score: 5
- +3 if urgent keywords detected (urgent, asap, emergency, deadline, etc.)
- +2 if from VIP domain (ceo@, founder@, admin@, security@, billing@)
- +2 if marked as [Priority]
- +1 if contains questions
- +1 if personalized (contains "you" or "your")
- -3 if spam indicators detected (unsubscribe, click here, winner, prize)
- Capped at 10

**Important Email** = score ≥ 7

## Database Schema

Emails are stored in the `email_history` table:

```sql
CREATE TABLE email_history (
    id TEXT PRIMARY KEY,
    contact_id TEXT,              -- Foreign key to contacts
    from_address TEXT NOT NULL,
    from_name TEXT,
    to_address TEXT NOT NULL,
    subject TEXT NOT NULL,
    body_text TEXT,               -- Plain text body
    body_html TEXT,               -- HTML body
    message_date INTEGER NOT NULL,
    message_type INTEGER NOT NULL, -- 1=received, 2=sent, 3=draft
    message_id TEXT,              -- Email Message-ID header
    in_reply_to TEXT,             -- Threading support
    email_references TEXT,        -- Threading support
    read_status INTEGER DEFAULT 0,
    has_attachments INTEGER DEFAULT 0,
    folder TEXT,

    -- AI Analysis
    importance_score INTEGER,     -- 0-10 scale
    is_important INTEGER DEFAULT 0, -- score >= 7
    is_urgent INTEGER DEFAULT 0,
    ai_summary TEXT,              -- AI-generated summary
    ai_analysis TEXT,             -- Full analysis JSON
    ai_analyzed_at TEXT,

    -- Metadata
    imported_at TEXT NOT NULL,
    imported_by TEXT NOT NULL,    -- 'imap_sync', 'smtp_send', etc.
    source_server TEXT,

    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL
);
```

Full-text search is available via the `email_history_fts` virtual table.

## Testing

### Test Email Receiving (with real IMAP)

1. Configure IMAP environment variables
2. Send an email to your configured email address
3. Watch the worker logs:
```bash
📬 Checking for new emails...
📨 Found 1 new email(s)
📧 Processing email from: sender@example.com
   Subject: Test Email
✨ Auto-created contact for sender@example.com
✅ Email stored in database: <uuid>
🤖 Analyzing email importance: Test Email
```

4. Check the database:
```bash
sqlite3 data/contacts.db "SELECT from_address, subject, importance_score FROM email_history ORDER BY message_date DESC LIMIT 5"
```

### Test Email Sending (with real SMTP)

```bash
# Example: Send via CLI (if CLI support added)
./target/release/sagenscontact email send \
  --to recipient@example.com \
  --subject "Test from SagensContact" \
  --body "This is a test email"
```

Or programmatically via the communication queue.

## Security Considerations

**⚠️ Alpha Release Limitations:**
- Passwords stored in plaintext environment variables
- No OAuth2 support yet
- No encryption at rest for email content
- Use app-specific passwords when possible
- Consider using a dedicated email account for testing

**Production Recommendations (Beta):**
- Use OAuth2 for Gmail/Outlook
- Encrypt credentials in config files
- Use TLS for all connections (already enabled)
- Implement email encryption (PGP/S/MIME)
- Add rate limiting and spam detection

## Troubleshooting

### "IMAP login failed"
- Verify username and password are correct
- For Gmail: Ensure you're using an app password, not your Google password
- Check that IMAP is enabled in your email provider settings
- Verify firewall allows outbound connections to port 993

### "SMTP send failed"
- Verify SMTP credentials
- Check SMTP_PORT (587 for STARTTLS, 465 for SSL)
- Ensure your email provider allows SMTP access
- Check for rate limits or sending restrictions

### "No new emails" but emails exist
- IMAP only fetches UNSEEN (unread) emails
- Mark emails as unread in your email client to test
- Check that the folder is "INBOX" (default folder)

### "Failed to parse email"
- Some email formats may not parse correctly
- Check worker logs for specific parsing errors
- Consider reporting malformed email samples as issues

## Next Steps

- **Email Reply**: Reply to emails from within SagensContact (planned for beta)
- **Email Templates**: Create reusable email templates
- **Email Scheduling**: Schedule emails for future sending
- **Email Folders**: Support multiple IMAP folders beyond INBOX
- **OAuth2**: Use OAuth2 instead of passwords (more secure)
- **Attachments**: Handle email attachments (currently text only)

## API Integration

Once emails are in the database, you can access them via:
- CLI: `sagenscontact email list`, `sagenscontact email search "keyword"`
- Web UI: Email history tab in contact details
- REST API: `GET /api/emails`, `GET /api/emails/:id`

## Support

For issues or questions:
- GitHub Issues: https://github.com/robcapo/sagenscontact/issues
- Documentation: See `alpha/ARCHITECTURE.md` for implementation details
