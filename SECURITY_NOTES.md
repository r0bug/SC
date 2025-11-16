# Security Notes

## ⚠️ Alpha Security Warnings

This alpha release has the following security limitations:

### 1. Plaintext Credential Storage

**Issue:** Legacy setups still rely on `config/credentials.toml` in plaintext, but an encrypted vault is now available.

**Risk:** Anyone with filesystem access can read API keys, database passwords, SMTP credentials, etc.

**Mitigation (Beta):**
- macOS: Integrate with Keychain via `security` command
- Linux: Use GNOME Keyring or KWallet
- Cross-platform: Support HashiCorp Vault
- Environment variables for containerized deployments

**Current Workaround:**
- Prefer the encrypted vault (`secure_vault` + `config/credentials.env`) with `SAGENSCONTACT_VAULT_FILE` and `SAGENSCONTACT_VAULT_KEY` set.
- Only fall back to plaintext TOML for local development and guard the file with `chmod 600` if you must keep it around.

### 2. No Authentication/Authorization

**Issue:** Sync service has no auth layer. Any client can access any endpoint.

**Risk:** Unauthorized data access, modification, deletion.

**Mitigation (Beta):**
- Implement JWT-based authentication
- Add per-user database isolation
- Enforce RBAC for sharing features
- Add API key authentication for service-to-service calls

**Current Workaround:**
- Run sync service on localhost only
- Use firewall rules to block external access
- Deploy in trusted network environments only

### 3. No Encryption at Rest

**Issue:** SQLite database stores all data in plaintext.

**Risk:** Anyone with filesystem access can read contacts, notes, communication history.

**Mitigation (Beta):**
- Enable SQLCipher for encrypted SQLite
- Encrypt sensitive fields (email, phone) with application-level encryption
- Use OS-level full-disk encryption as baseline

**Current Workaround:**
- Use full-disk encryption (FileVault on macOS, LUKS on Linux)
- Avoid storing highly sensitive information in alpha

### 4. No TLS/HTTPS

**Issue:** Sync service uses unencrypted HTTP and WebSocket.

**Risk:** Network sniffing can intercept all API calls, including sensitive contact data.

**Mitigation (Beta):**
- Enable TLS for Axum server
- Use wss:// for WebSocket connections
- Generate self-signed certs for development, Let's Encrypt for production

**Current Workaround:**
- Use sync service on localhost only
- Use VPN or SSH tunnel for remote access

### 5. Mock External Services

**Issue:** Email, SMS, social, AI adapters return fake responses.

**Risk:** Users may think communications were actually sent.

**Mitigation (Beta):**
- Implement real SMTP for email
- Integrate Twilio/AWS SNS for SMS
- Add OAuth2 for social platforms (Twitter, LinkedIn)
- Connect to real Segmind API

**Current Workaround:**
- Clearly log "[MOCK]" prefix in all mock adapter outputs
- Document mock behavior in README

### 6. No Input Validation

**Issue:** Limited validation on API inputs.

**Risk:** SQL injection (mitigated by sqlx parameterization), XSS in web UI, path traversal for attachments.

**Mitigation (Beta):**
- Add comprehensive input validation with validator crate
- Sanitize HTML content in notes
- Validate file upload extensions and MIME types
- Stream every upload through ClamAV (INSTREAM) and block infected files

**Current Workaround:**
- ClamAV integration is live; set `VIRUS_SCANNER_ENABLED=true` and point `CLAMAV_SOCKET_PATH` at your `clamd` socket to enforce scanning.
- For purely local testing, you can toggle `VIRUS_SCANNER_ENABLED=false` to fall back to the mock scanner.

### 7. No Audit Logging

**Issue:** No logging of security-relevant events.

**Risk:** Cannot detect or investigate unauthorized access, data breaches.

**Mitigation (Beta):**
- Add audit log table for all CRUD operations
- Log authentication attempts
- Track share invite acceptances
- Implement log aggregation for production

**Current Workaround:**
- Review application logs manually
- Monitor database file access times

### 8. Dependency Vulnerabilities

**Issue:** Rust and npm dependencies may have known CVEs.

**Risk:** Exploitable vulnerabilities in third-party code.

**Mitigation (Beta):**
- Run `cargo audit` and `pnpm audit` in CI
- Enable Dependabot or Renovate for automated updates
- Pin dependency versions with security patches

**Current Workaround:**
- Manually run `cargo audit` before releases
- Keep Rust toolchain updated

## Threat Model (Beta/Production)

### Assets
- Contact data (PII: names, emails, phones, addresses)
- Notes and attachments (potentially sensitive documents)
- Communication history
- Share permissions

### Threats
1. **Unauthorized Access**: Attacker gains access to database file
2. **Network Interception**: Man-in-the-middle attack on sync traffic
3. **Credential Theft**: API keys or passwords stolen from config files
4. **Malicious Attachments**: Virus or malware uploaded via note attachments
5. **Account Takeover**: Attacker compromises user credentials (beta+)
6. **Data Exfiltration**: Attacker syncs contact data to external server

### Controls
- [ ] Encryption at rest (SQLCipher)
- [ ] Encryption in transit (TLS)
- [ ] Secure credential storage (OS keychain)
- [ ] Authentication (JWT, OAuth2)
- [ ] Authorization (RBAC)
- [ ] Input validation and sanitization
- [ ] Virus scanning on file uploads
- [ ] Audit logging
- [ ] Rate limiting
- [ ] Security headers (CSP, HSTS, X-Frame-Options)

## Compliance Considerations

For production use, consider:
- **GDPR**: Right to access, right to deletion, data portability, consent management
- **CCPA**: Privacy policy, opt-out mechanisms
- **HIPAA**: If handling health information, requires encryption, audit logs, BAA
- **SOC 2**: For hosted/SaaS, requires security controls documentation

## Security Checklist for Beta

- [ ] Remove plaintext credential files
- [ ] Implement secure credential vault
- [ ] Add user authentication
- [ ] Enable TLS for sync service
- [ ] Encrypt sensitive database fields
- [ ] Add input validation
- [ ] Implement virus scanning
- [ ] Add audit logging
- [ ] Run security audit / penetration test
- [ ] Document incident response plan

## Reporting Security Issues

For security concerns, contact: [security@sagenscontact.example] (placeholder for alpha)

Do not disclose security issues publicly until coordinated disclosure timeline.
