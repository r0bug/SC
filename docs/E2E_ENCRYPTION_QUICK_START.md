# E2E Encryption Quick Start Guide

This is a quick reference for using the End-to-End Encryption feature in SagensContact.

## For Developers

### Client-Side Usage (TypeScript)

```typescript
import { CryptoClient, EncryptedEntityClient } from '$lib/crypto';

// 1. Initialize crypto client with user password
const crypto = new CryptoClient();
const salt = await crypto.initializeKey('user-password-123');
// Store salt in localStorage or database for future sessions

// 2. Create API client
const client = new EncryptedEntityClient('http://localhost:3000', crypto);

// 3. Store encrypted contact
const contact = {
  id: crypto.randomUUID(),
  first_name: 'John',
  last_name: 'Doe',
  email: 'john@example.com',
  phone: '+1234567890'
};

const response = await client.storeEntity('user-1', 'contact', contact);
console.log('Stored:', response.id);

// 4. Retrieve and decrypt
const decrypted = await client.getEntity<typeof contact>('user-1', response.id);
console.log('Name:', decrypted.first_name, decrypted.last_name);

// 5. List all contacts (encrypted metadata only)
const entities = await client.listEntities('user-1', 'contact');
console.log('Total contacts:', entities.length);

// 6. Decrypt each entity
for (const entity of entities) {
  const blob = crypto.deserializeBlob(entity.encrypted_data);
  const contact = await crypto.decryptEntity<Contact>(blob);
  console.log('Contact:', contact.first_name, contact.last_name);
}

// 7. Update contact
contact.phone = '+0987654321';
await client.updateEntity('user-1', response.id, contact);

// 8. Delete contact (soft delete)
await client.deleteEntity('user-1', response.id);
```

### Server-Side (Rust)

The server never decrypts data - it only stores and retrieves encrypted blobs.

```rust
// Encryption routes are automatically mounted at /api/encrypted/*
// See: crates/sync_service/src/encryption.rs
```

## For System Administrators

### Installation

1. **Run database migration**:
   ```bash
   cd /home/robug/Projects/sagenscontact/alpha
   sqlx migrate run
   ```

2. **Verify tables exist**:
   ```bash
   sqlite3 data/sagenscontact.db "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'encrypt%';"
   ```
   Expected output:
   ```
   encryption_keys
   encrypted_entities
   key_rotation_log
   ```

### Migration Tool

To encrypt existing plaintext data:

```bash
# Build and run migration tool
cargo run --release --bin encrypt_existing

# Interactive prompts
Enter user_id: user-1
Enter master password: ********

# Output
Migrating contacts....... Done! (142 contacts)
Migrating notes... Done! (37 notes)
Migrating projects... Done! (12 projects)
Migrating calendar events... Done! (28 calendar events)
Total entities: 219
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/encrypted/:user_id` | Store encrypted entity |
| GET | `/api/encrypted/:user_id/:entity_id` | Get encrypted entity |
| GET | `/api/encrypted/:user_id?entity_type=contact` | List entities |
| PUT | `/api/encrypted/:user_id/:entity_id` | Update entity |
| DELETE | `/api/encrypted/:user_id/:entity_id` | Soft delete entity |

### Testing

```bash
# Store encrypted contact
curl -X POST http://localhost:3000/api/encrypted/user-1 \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "contact",
    "encrypted_data": "nonce_b64.ciphertext_b64",
    "checksum": "sha256_hex"
  }'

# List all contacts
curl http://localhost:3000/api/encrypted/user-1?entity_type=contact
```

## Security Notes

1. **Passwords are never sent to the server** - only derived encryption keys are used locally
2. **Server cannot decrypt data** - it only stores encrypted blobs
3. **Lost password = lost data** - zero-knowledge encryption means no password recovery
4. **Use strong passwords** - minimum 12 characters, mix of types
5. **Enable HTTPS in production** - prevent man-in-the-middle attacks

## Troubleshooting

### Error: Checksum verification failed

**Cause**: Data corruption or tampering
**Solution**: Restore from backup

### Error: Crypto key not initialized

**Cause**: Forgot to call `initializeKey()`
**Solution**: Always initialize before encryption/decryption

### Error: Invalid encrypted blob format

**Cause**: Malformed base64 or missing delimiter
**Solution**: Check serialization format: `<nonce_b64>.<ciphertext_b64>`

## Performance Tips

1. **Cache derived keys** - Don't re-derive on every operation
2. **Batch operations** - Encrypt multiple entities before sending to server
3. **Use web workers** - Offload encryption to background threads
4. **Lazy decryption** - Decrypt only when user views entity

## Further Reading

- Full documentation: `/home/robug/Projects/sagenscontact/alpha/docs/E2E_ENCRYPTION.md`
- Implementation files: See `E2E_ENCRYPTION_FILES_SUMMARY.md`
- Security considerations: Section 10 in main documentation
