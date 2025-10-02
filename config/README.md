# Configuration

## Setup

1. Copy `credentials.toml.example` to `credentials.toml`
2. Replace placeholder values with actual credentials (for production use)

## Alpha Version Notice

⚠️ **IMPORTANT**: This alpha version uses placeholder credentials stored in plain text files.
These are NOT SECURE and should only be used for development and testing.

### Current Limitations

- Credentials stored in plain TOML files
- No encryption at rest
- No secure credential vault integration
- File-based authentication only

### Future Roadmap (Beta+)

- Integration with system keychain (macOS Keychain, GNOME Keyring, Windows Credential Store)
- Vault support (HashiCorp Vault, AWS Secrets Manager)
- Encrypted credential storage
- Multi-factor authentication support
- OAuth2/OIDC integration for social platforms

## Configuration Files

### credentials.toml
Main credential store with sections:
- `[database]` - Database connection strings
- `[segmind]` - AI service credentials
- `[email]` - SMTP configuration
- `[sms]` - SMS provider settings
- `[social]` - Social media API credentials
- `[storage]` - File storage configuration (local/MinIO)
- `[sync]` - Sync service endpoints

## Migration to Secure Vault

When migrating to beta, follow these steps:

1. Choose vault backend (keychain/Vault/cloud secrets)
2. Migrate credentials using the provided migration script (TBD)
3. Update application configuration to use vault URLs instead of file paths
4. Remove credentials.toml from filesystem
5. Verify secure credential access

## Environment Variables

For CI/CD and containerized deployments, credentials can be overridden via environment variables:

```bash
SAGENSCONTACT_DATABASE_URL="sqlite:./contacts.db"
SAGENSCONTACT_SEGMIND_API_KEY="your_key_here"
```