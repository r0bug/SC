# Testing

## Test Strategy

SagensContact uses a multi-layered testing approach:

1. **Unit Tests**: Core domain logic and repository functions
2. **Integration Tests**: Database operations and API endpoints
3. **E2E Tests**: Full workflow testing via CLI and web UI (planned)
4. **Manual Tests**: Acceptance criteria for alpha release

**Phase 6 Coverage**: Comprehensive tests for attachment upload/download/scan flows, AI interaction logging with caching/retry tracking, and search history enrichment with privacy mode.

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

### Run with Coverage (requires cargo-tllvm-cov)

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
| local_store/attachment | ✅ (8 tests) | ✅ | - | ✅ |
| local_store/ai_interaction | ✅ (8 tests) | ✅ | - | ✅ |
| sync_service | ✅ | ✅ | ⏳ | ✅ |
| communication_queue | ✅ | ✅ | - | ✅ |
| ai_middleware | ✅ | - | - | ✅ |
| ai_middleware/logging | ✅ | ✅ | - | ✅ |
| cli_client | ✅ | ✅ | ✅ | ✅ |
| desktop | ⏳ | ⏳ | ⏳ | ⏳ |
| web | ⏳ | ⏳ | ⏳ | ⏳ |

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

## Phase 6: Attachment & AI Interaction Tests

### Attachment System Tests (`crates/local_store/src/repositories/attachment_tests.rs`)

**8 comprehensive test cases covering:**

1. **`test_attachment_create_and_retrieve`** - Full CRUD with all metadata fields (checksum, scan_status, encrypted flag)
2. **`test_attachment_list_by_entity`** - Entity association and ordering (3 attachments, ordered by created_at DESC)
3. **`test_attachment_scan_status_tracking`** - State machine: Pending → Infected with scan_details
4. **`test_attachment_checksum_integrity`** - SHA-256 checksum storage and validation
5. **`test_attachment_encryption_flag`** - Encryption-at-rest tracking with AES-256 metadata
6. **`test_attachment_delete`** - Deletion and verification
7. **`test_attachment_multiple_entity_types`** - Polymorphic attachment support across entities

**Run attachment tests:**
```bash
cargo test -p local_store attachment_tests
```

### AI Interaction Tests (`crates/local_store/src/repositories/ai_interaction_tests.rs`)

**8 comprehensive test cases covering:**

1. **`test_ai_interaction_logging`** - Interaction recording with prompt, response, confidence, model
2. **`test_ai_interaction_feedback`** - User feedback loop (helpful/applied) with timestamps
3. **`test_ai_interaction_list_by_user`** - User-specific history (5 interactions)
4. **`test_ai_interaction_list_by_entity`** - Entity-context filtering (3 interactions for one contact)
5. **`test_ai_interaction_cache_tracking`** - Cache hit metrics in metadata JSON
6. **`test_ai_interaction_retry_tracking`** - Retry attempts and backoff durations tracking
7. **`test_ai_interaction_delete`** - Privacy/GDPR deletion support
8. **`test_ai_interaction_recent_list`** - Cross-user recent interactions (10 created, 5 returned)

**Run AI interaction tests:**
```bash
cargo test -p local_store ai_interaction_tests
```

## Integration Tests

Integration tests are located in `crates/*/tests/` directories and test multi-component interactions.

Example:
```rust
#[tokio::test]
async fn test_import_and_search_workflow() {
    // Setup: Create store and import sample data
    let store = LocalStore::new("sqlite::memory:").await.unwrap();
    import_csv("sample_data/contacts.csv", &store).await.unwrap();

    // Test: Search for imported contact
    let repo = ContactRepository::new(store.pool());
    let results = repo.search("John").await.unwrap();
    assert!(results.len() > 0);
    assert_eq!(results[0].first_name, "John");
}
```

## E2E Tests (Planned)

### CLI E2E Script
```bash
#!/bin/bash
# scripts/cli_e2e_test.sh

set -e

# Setup
rm -f data/test.db
export DATABASE_URL="sqlite:data/test.db"

# Import
./target/release/sagenscontact import --csv sample_data/contacts.csv

# List
OUTPUT=$(./target/release/sagenscontact list)
echo "$OUTPUT" | grep "John Doe"

# Search
OUTPUT=$(./target/release/sagenscontact search "john")
echo "$OUTPUT" | grep "john.doe@example.com"

# Add
./target/release/sagenscontact add "Test" "User" --email test@example.com

# Search for added contact
OUTPUT=$(./target/release/sagenscontact search "Test User")
echo "$OUTPUT" | grep "test@example.com"

echo "CLI E2E tests passed!"
```

### Web E2E with Playwright (TODO)
```typescript
// apps/web/tests/e2e/import-share.spec.ts
import { test, expect } from '@playwright/test';

test('import and share workflow', async ({ page }) => {
  await page.goto('/');

  // Import contacts
  await page.click('button:has-text("Import")');
  await page.setInputFiles('input[type="file"]', 'sample_data/contacts.csv');
  await page.click('button:has-text("Upload")');

  // Verify import
  await expect(page.locator('text=John Doe')).toBeVisible();

  // Create share invite
  await page.click('text=John Doe');
  await page.click('button:has-text("Share")');
  await page.fill('input[name="email"]', 'recipient@example.com');
  await page.click('button:has-text("Send Invite")');

  // Verify success
  await expect(page.locator('text=Share invite sent')).toBeVisible();
});
```

## Manual Test Checklist (Alpha Acceptance)

### ✅ Core Functionality

- [ ] Build succeeds: `cargo build --release`
- [ ] CLI commands work:
  - [ ] `import --csv sample_data/contacts.csv`
  - [ ] `list` shows imported contacts
  - [ ] `search john` finds John Doe
  - [ ] `add "New" "User" --email new@example.com`
  - [ ] `note <contact_id> "Title" "Content"`
  - [ ] `communicate <contact_id> email "Hello"`
  - [ ] `share contact <contact_id> recipient@example.com`
  - [ ] `suggest <contact_id>` returns AI suggestions

### ✅ Sync Service

- [ ] Service starts: `cargo run --bin sync_service`
- [ ] Health check: `curl http://localhost:3000/health` returns "OK"
- [ ] API endpoints respond (use curl or Postman):
  - [ ] `GET /api/contacts` returns JSON array
  - [ ] `POST /api/contacts` creates new contact
  - [ ] `GET /api/tags` returns tags
  - [ ] `POST /api/communication` queues communication

### ✅ Import Workflow

- [ ] CSV import: All 5 sample contacts imported
- [ ] vCard import: Sarah Connor and James Martinez imported
- [ ] SMS import: Conversation messages logged (mock)

### ✅ Communication Queue

- [ ] Queued email shows "[MOCK] Email sent successfully"
- [ ] Queued SMS shows "[MOCK] SMS sent successfully"
- [ ] Failed communication (message contains "test-fail") logs retry
- [ ] After 3 retries, status changes to Failed

### ✅ AI Suggestions

- [ ] Suggestion for contact with "organization" field suggests tags
- [ ] Confidence scores are reasonable (0.7-0.9 range)
- [ ] Suggestions are deterministic (same input = same output)

### ✅ Attachment Management (Phase 6)

- [ ] Upload attachment via web UI (< 100MB)
- [ ] Attachment appears in list with correct filename and size
- [ ] Scan status badge shows "Clean" (mock scanner)
- [ ] Download attachment retrieves original file with matching checksum
- [ ] Delete attachment removes from database and UI
- [ ] Multiple attachments per entity display correctly

### ✅ AI Interaction Logging (Phase 6)

- [ ] AI suggestion generates logged interaction in `ai_interactions` table
- [ ] Interaction includes prompt, response, confidence, model name
- [ ] Cache hits tracked in metadata JSON
- [ ] Retry attempts logged with backoff durations
- [ ] User feedback (helpful/not helpful) persists
- [ ] Applied suggestions marked with timestamp

### ✅ Search History Enrichment (Phase 6)

- [ ] Search stores result_ids array of returned contact UUIDs
- [ ] Recent searches widget displays last 10 searches
- [ ] Privacy mode enabled: search executes but NOT stored in history
- [ ] Privacy mode disabled: search appears in recent searches
- [ ] Metadata JSON extensible for future features

### ✅ Sharing

- [ ] Share invite created successfully
- [ ] Invite stored in database with correct entity_type and permissions
- [ ] (Future) Recipient receives email notification

### ✅ Data Persistence

- [ ] Restart CLI, contacts still present
- [ ] Edit contact, changes persist after app restart
- [ ] Delete contact, removed from subsequent list commands

### ✅ Error Handling

- [ ] Invalid UUID shows helpful error message
- [ ] Missing database file auto-creates on first run
- [ ] Duplicate contact email (if unique constraint) shows conflict error

## CI/CD Pipeline (GitHub Actions)

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
          toolchain: 1.75
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

All test data is in `sample_data/` directory. See `sample_data/README.md` for details.

## Performance Testing (Future)

- Load testing with wrk or artillery
- Database query optimization with EXPLAIN QUERY PLAN
- Memory profiling with valgrind or heaptrack
- Benchmark suite with criterion.rs

## Security Testing (Future)

- OWASP Top 10 checks
- Dependency vulnerability scanning (cargo audit)
- Penetration testing on sync service endpoints
- Fuzzing inputs with cargo-fuzz