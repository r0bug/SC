# Testing

## Test Strategy

SagensContact uses a multi-layered testing approach:

1. **Unit Tests**: Core domain logic and repository functions
2. **Integration Tests**: Database operations and API endpoints
3. **E2E Tests**: CLI script and Playwright web tests
4. **Manual Tests**: Acceptance criteria verification

## Running Tests

### Rust Unit Tests

```bash
cd alpha
cargo test
```

### Rust Integration Tests

```bash
cargo test --test integration
```

### CLI E2E Tests

```bash
./scripts/cli_e2e_test.sh
```

### Web E2E Tests (Playwright)

```bash
cd apps/web
pnpm install
pnpm test        # Run tests
pnpm test:ui     # Interactive mode
```

### Run with Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### Lint and Format

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Test Matrix

| Component | Unit | Integration | E2E | Manual |
|-----------|------|-------------|-----|--------|
| core_domain | ✅ | - | - | - |
| local_store | ✅ | ✅ | - | - |
| local_store/attachment | ✅ | ✅ | - | ✅ |
| local_store/ai_interaction | ✅ | ✅ | - | ✅ |
| sync_service | ✅ | ✅ | ✅ | ✅ |
| communication_queue | ✅ | ✅ | - | ✅ |
| ai_middleware | ✅ | - | - | ✅ |
| import_service | ✅ | ✅ | - | ✅ |
| cli_client | ✅ | ✅ | ✅ | ✅ |
| desktop | ✅ | - | ⏳ | ✅ |
| web | ✅ | ✅ | ✅ | ✅ |

Legend: ✅ Implemented | ⏳ TODO | - N/A

## Unit Test Examples

### Domain Entity Test
```rust
#[test]
fn test_contact_creation() {
    let contact = Contact {
        id: Uuid::new_v4(),
        first_name: "John".to_string(),
        last_name: Some("Doe".to_string()),
        email: Some("john@example.com".to_string()),
        // ...
    };
    assert_eq!(contact.first_name, "John");
}
```

### Repository Test
```rust
#[tokio::test]
async fn test_contact_crud() {
    let store = LocalStore::new("sqlite::memory:").await.unwrap();
    let repo = ContactRepository::new(store.pool());

    let contact = create_test_contact();
    repo.create(&contact).await.unwrap();

    let fetched = repo.get_by_id(contact.id).await.unwrap();
    assert_eq!(fetched.id, contact.id);
}
```

## Attachment System Tests

Located in `crates/local_store/src/repositories/attachment_tests.rs`:

1. **`test_attachment_create_and_retrieve`** - Full CRUD with metadata
2. **`test_attachment_list_by_entity`** - Entity association and ordering
3. **`test_attachment_scan_status_tracking`** - Pending → Clean/Infected
4. **`test_attachment_checksum_integrity`** - SHA-256 validation
5. **`test_attachment_encryption_flag`** - Encryption-at-rest tracking
6. **`test_attachment_delete`** - Deletion verification
7. **`test_attachment_multiple_entity_types`** - Polymorphic support

```bash
cargo test -p local_store attachment_tests
```

## AI Interaction Tests

Located in `crates/local_store/src/repositories/ai_interaction_tests.rs`:

1. **`test_ai_interaction_logging`** - Recording with metadata
2. **`test_ai_interaction_feedback`** - User feedback loop
3. **`test_ai_interaction_list_by_user`** - User-specific history
4. **`test_ai_interaction_list_by_entity`** - Entity-context filtering
5. **`test_ai_interaction_cache_tracking`** - Cache hit metrics
6. **`test_ai_interaction_retry_tracking`** - Retry attempts tracking
7. **`test_ai_interaction_delete`** - GDPR deletion support
8. **`test_ai_interaction_recent_list`** - Recent interactions query

```bash
cargo test -p local_store ai_interaction_tests
```

## Web E2E Tests

Located in `apps/web/tests/`:

### web-flows.test.ts
- Authentication flows
- Contact CRUD operations
- Search functionality
- Notes management

### api-integration.test.ts
- API health checks
- Contact import flow (CSV, vCard, social)
- Communication tabs (SMS, Email)
- Attachment management
- WebSocket real-time updates
- Error handling
- Performance benchmarks
- Responsive design (mobile/tablet)
- Accessibility (headings, labels, keyboard nav)

```bash
cd apps/web
pnpm test
```

## Manual Test Checklist

### Core Functionality

- [ ] Build succeeds: `cargo build --release`
- [ ] CLI commands work:
  - [ ] `import --csv sample_data/contacts.csv`
  - [ ] `import --vcard sample_data/contacts.vcf`
  - [ ] `import --json linkedin_connections.json`
  - [ ] `list` shows imported contacts
  - [ ] `search john` finds contacts
  - [ ] `add "New" "User" --email new@example.com`
  - [ ] `note <contact_id> "Title" "Content"`
  - [ ] `communicate <contact_id> email "Hello"`
  - [ ] `suggest <contact_id>` returns suggestions

### Sync Service

- [ ] Service starts: `cargo run --bin sync_service`
- [ ] Health check: `curl http://localhost:3000/health`
- [ ] API endpoints respond:
  - [ ] `GET /api/contacts`
  - [ ] `POST /api/contacts`
  - [ ] `GET /api/tags`
  - [ ] `POST /api/attachments` (multipart upload)
  - [ ] `GET /api/import/preview`

### Import Workflow

- [ ] CSV import with field mapping preview
- [ ] vCard import
- [ ] LinkedIn connections export import
- [ ] Twitter/X archive import
- [ ] Facebook friends.json import
- [ ] Instagram JSON import

### Communication Queue

- [ ] Email queued (with SMTP config, sends real email)
- [ ] SMS queued (with Twilio config, sends real SMS)
- [ ] Without config, logs "[MOCK]" prefix
- [ ] Retry logic on failure

### AI Suggestions

- [ ] With `SEGMIND_API_KEY`: real AI suggestions
- [ ] Without API key: deterministic mock suggestions
- [ ] Interaction logged to database
- [ ] Cache prevents duplicate API calls

### Attachment Management

- [ ] Upload attachment (< 100MB)
- [ ] Virus scan status shows (Clean/Pending)
- [ ] Download attachment
- [ ] Delete attachment

### Data Persistence

- [ ] Restart application, data persists
- [ ] Edit entity, changes saved
- [ ] Delete entity, removed from list

## CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.83
          override: true
      - name: Run tests
        run: cargo test --all
      - name: Lint
        run: cargo clippy -- -D warnings
      - name: Format check
        run: cargo fmt --check

  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v3
      - name: Build release
        run: cargo build --release
      - name: Run E2E script
        run: ./scripts/cli_e2e_test.sh
```

## Test Data

Test data is in `sample_data/` directory:
- `contacts.csv` - Sample contacts
- `contacts.vcf` - vCard samples
- `sms_export.json` - SMS conversation samples

## Performance Testing

```bash
# Database query analysis
sqlite3 data/sagenscontact.db "EXPLAIN QUERY PLAN SELECT * FROM contacts WHERE email LIKE '%@%'"

# Load testing (requires wrk)
wrk -t4 -c100 -d30s http://localhost:3000/api/contacts
```

## Security Testing

```bash
# Dependency audit
cargo audit

# npm audit
cd apps/web && pnpm audit
```
