# Security Notes

## Security Status

### Implemented Security Features

#### 1. Encrypted Credential Vault
**Status:** Implemented

The `secure_vault` crate provides AES-256-GCM encrypted credential storage:
```bash
# Encrypt credentials
cargo run -p secure_vault --features cli --bin vault_tool -- \
  encrypt --input config/credentials.env --output config/credentials.vault \
  --key "your-master-key"

# Use at runtime
export SAGENSCONTACT_VAULT_FILE="config/credentials.vault"
export SAGENSCONTACT_VAULT_KEY="your-master-key"
```

#### 2. Audit Logging
**Status:** Implemented

Comprehensive audit logging for security events:
- All CRUD operations on entities
- Authentication attempts (login success/failure)
- Share invite creation and acceptance
- File upload/download events
- ACL changes

Logs stored in `audit_logs` table with user_id, action, entity_type, entity_id, and metadata.

#### 3. ACL Enforcement
**Status:** Implemented

Fine-grained access control on all API routes:
- Resource-level permissions (Read, Write, Delete, Share, Admin)
- Per-entity ACL enforcement
- Owner and shared access checks

#### 4. Input Validation
**Status:** Implemented

Comprehensive validation:
- File upload: type, size, extension, MIME type verification
- Password strength requirements
- Email format validation
- Phone number format validation
- Path traversal prevention

#### 5. Virus Scanning
**Status:** Implemented

ClamAV integration for file uploads:
```bash
# Enable ClamAV scanning
export VIRUS_SCANNER_ENABLED=true
export CLAMAV_SOCKET_PATH=/var/run/clamav/clamd.sock
export VIRUS_SCANNER_STRICT=true  # Fail if ClamAV unavailable
```

Files are streamed through ClamAV's INSTREAM protocol. Infected files are automatically rejected and deleted.

#### 6. External Service Integration
**Status:** Implemented (configurable)

Real external services available with configuration:
- **Email**: SMTP integration (`SMTP_HOST`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM`)
- **SMS**: Twilio integration (`TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_PHONE_NUMBER`)
- **AI**: Segmind API (`SEGMIND_API_KEY`)

Without configuration, services operate in fallback mode (logging only).

---

## Remaining Security Gaps

### 1. No TLS/HTTPS
**Issue:** Sync service uses unencrypted HTTP and WebSocket.

**Risk:** Network sniffing can intercept API calls.

**Mitigation:**
- Run sync service behind a reverse proxy (nginx, Caddy) with TLS
- Use localhost-only binding for development
- Use VPN or SSH tunnel for remote access

### 2. No Encryption at Rest
**Issue:** SQLite database stores data in plaintext.

**Risk:** Filesystem access exposes all contact data.

**Mitigation:**
- Use OS-level full-disk encryption (FileVault, LUKS)
- SQLCipher integration planned for future release

### 3. Limited Authentication
**Issue:** Single-user JWT authentication only.

**Risk:** No multi-user isolation, no OAuth2 support.

**Mitigation:**
- Run in trusted environment
- Full authentication system planned for beta

---

## Security Configuration

### Recommended Production Setup

```bash
# 1. Use encrypted vault for credentials
export SAGENSCONTACT_VAULT_FILE="config/credentials.vault"
export SAGENSCONTACT_VAULT_KEY="strong-master-key"

# 2. Enable virus scanning
export VIRUS_SCANNER_ENABLED=true
export CLAMAV_SOCKET_PATH=/var/run/clamav/clamd.sock
export VIRUS_SCANNER_STRICT=true

# 3. Run behind TLS reverse proxy
# Example nginx config:
# server {
#     listen 443 ssl;
#     ssl_certificate /path/to/cert.pem;
#     ssl_certificate_key /path/to/key.pem;
#     location / {
#         proxy_pass http://127.0.0.1:3000;
#     }
# }

# 4. Bind to localhost only
export BIND_ADDRESS=127.0.0.1:3000
```

---

## Threat Model

### Assets
- Contact data (PII: names, emails, phones, addresses)
- Notes and attachments (potentially sensitive documents)
- Communication history
- Share permissions
- Credentials and API keys

### Threats & Mitigations

| Threat | Mitigation |
|--------|------------|
| Unauthorized API access | ACL enforcement, audit logging |
| Credential theft | Encrypted vault |
| Malicious file upload | ClamAV scanning, file type validation |
| SQL injection | SQLx parameterized queries |
| XSS attacks | Input sanitization |
| Network interception | TLS via reverse proxy |
| Database file theft | Full-disk encryption (OS level) |

---

## Security Controls Checklist

- [x] Encrypted credential storage (secure_vault)
- [x] ACL enforcement on all routes
- [x] Audit logging
- [x] Input validation and sanitization
- [x] Virus scanning (ClamAV)
- [x] File upload validation (type, size, extension)
- [x] Password strength requirements
- [x] CORS configuration
- [ ] TLS/HTTPS (use reverse proxy)
- [ ] Encryption at rest (SQLCipher)
- [ ] Multi-user authentication (OAuth2)
- [ ] Rate limiting
- [ ] Security headers (CSP, HSTS)

---

## Compliance Considerations

For production use, consider:
- **GDPR**: Right to access, right to deletion, data portability
- **CCPA**: Privacy policy, opt-out mechanisms
- **HIPAA**: Encryption, audit logs, BAA (if handling health info)
- **SOC 2**: Security controls documentation

---

## Reporting Security Issues

For security concerns, please report responsibly:
1. Do not disclose publicly until fixed
2. Provide clear reproduction steps
3. Allow reasonable time for patching

---

## Development Security

```bash
# Run security audit on Rust dependencies
cargo audit

# Run security audit on npm dependencies
cd apps/web && pnpm audit

# Check for outdated dependencies
cargo outdated
```
