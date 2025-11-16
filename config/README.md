# Configuration

## Setup

### Encrypted Vault (Recommended)

1. Copy `credentials.env.example` to `credentials.env` and populate it with your real secrets in `KEY=value` format.
2. Run the vault utility to encrypt the file:

   ```bash
   cargo run -p secure_vault --features cli --bin vault_tool -- \
     encrypt --input config/credentials.env --output config/credentials.vault \
     --key "choose-a-strong-master-key"
   ```

3. Set the following environment variables for every process (sync service, worker, CLI, desktop app):

   ```bash
   export SAGENSCONTACT_VAULT_FILE="/path/to/config/credentials.vault"
   export SAGENSCONTACT_VAULT_KEY="choose-a-strong-master-key"
   ```

4. Start the application. The vault loader injects the decrypted values into the process environment before any configuration is read.

Need to audit the contents? Run:

```bash
cargo run -p secure_vault --features cli --bin vault_tool -- \
  decrypt --input config/credentials.vault --key "master-key"
```

### Legacy Plaintext Mode

1. Copy `credentials.toml.example` to `credentials.toml`
2. Replace placeholder values with actual credentials (development/test only)

## Alpha Version Notice

⚠️ **IMPORTANT**: This alpha version uses placeholder credentials stored in plain text files.
These are NOT SECURE and should only be used for development and testing.

### Current Capabilities

- Encrypted credential vault with PBKDF2 + AES-GCM using `secure_vault`
- Plaintext TOML configuration for compatibility (development only)
- Environment variable overrides for CI/CD

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
