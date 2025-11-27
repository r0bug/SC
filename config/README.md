# Configuration

## Encrypted Vault (Recommended)

The secure vault provides AES-256-GCM encrypted credential storage:

### Setup

1. Create credentials file from template:
   ```bash
   cp credentials.env.example credentials.env
   ```

2. Edit `credentials.env` with your real credentials:
   ```bash
   # Database
   DATABASE_URL=sqlite:./data/sagenscontact.db

   # Email (SMTP)
   SMTP_HOST=smtp.example.com
   SMTP_PORT=587
   SMTP_USER=user@example.com
   SMTP_PASSWORD=your-password
   SMTP_FROM=noreply@example.com

   # SMS (Twilio)
   TWILIO_ACCOUNT_SID=your-sid
   TWILIO_AUTH_TOKEN=your-token
   TWILIO_PHONE_NUMBER=+1234567890

   # AI (Segmind)
   SEGMIND_API_KEY=your-api-key

   # Storage (S3/MinIO)
   S3_ENDPOINT_URL=http://localhost:9000
   S3_ACCESS_KEY_ID=minioadmin
   S3_SECRET_ACCESS_KEY=minioadmin
   S3_BUCKET=attachments
   ```

3. Encrypt the file:
   ```bash
   cargo run -p secure_vault --features cli --bin vault_tool -- \
     encrypt --input config/credentials.env --output config/credentials.vault \
     --key "choose-a-strong-master-key"
   ```

4. Set environment variables for runtime:
   ```bash
   export SAGENSCONTACT_VAULT_FILE="$PWD/config/credentials.vault"
   export SAGENSCONTACT_VAULT_KEY="choose-a-strong-master-key"
   ```

5. Start the application. Credentials are decrypted and injected automatically.

### Decrypt for Audit

```bash
cargo run -p secure_vault --features cli --bin vault_tool -- \
  decrypt --input config/credentials.vault --key "master-key"
```

## Environment Variables

All services can be configured via environment variables:

### Database
```bash
DATABASE_URL=sqlite:./data/sagenscontact.db
# Or for PostgreSQL:
DATABASE_URL=postgres://user:pass@localhost/sagenscontact
```

### Email (SMTP)
```bash
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASSWORD=password
SMTP_FROM=noreply@example.com
```

### SMS (Twilio)
```bash
TWILIO_ACCOUNT_SID=your-sid
TWILIO_AUTH_TOKEN=your-token
TWILIO_PHONE_NUMBER=+1234567890
```

### AI (Segmind)
```bash
SEGMIND_API_KEY=your-api-key
```

### Virus Scanning (ClamAV)
```bash
VIRUS_SCANNER_ENABLED=true
CLAMAV_SOCKET_PATH=/var/run/clamav/clamd.sock
VIRUS_SCANNER_STRICT=true
```

### Storage (S3/MinIO)
```bash
S3_ENDPOINT_URL=http://localhost:9000
S3_REGION=us-east-1
S3_ACCESS_KEY_ID=minioadmin
S3_SECRET_ACCESS_KEY=minioadmin
S3_BUCKET=attachments
```

### Cache (Redis)
```bash
REDIS_URL=redis://localhost:6379
```

### Server
```bash
PORT=3000
BIND_ADDRESS=127.0.0.1:3000
RUST_LOG=info
```

## Legacy Plaintext Mode (Development Only)

For local development without encryption:

1. Copy template: `cp credentials.toml.example credentials.toml`
2. Edit with placeholder or test credentials
3. Protect the file: `chmod 600 credentials.toml`

**Warning:** Do not use plaintext credentials in production.

## Service Fallback Behavior

Without configuration, services operate in fallback mode:

| Service | Without Config |
|---------|---------------|
| Email | Logs to console |
| SMS | Logs to console |
| AI | Returns mock suggestions |
| Virus Scan | Basic file check only |
| Storage | Local filesystem |
| Cache | In-memory (moka) |

This allows development and testing without external service dependencies.
