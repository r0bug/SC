# Workflow Demonstrations

## Demo Scenario: Military Artifact Sharing

This document walks through the alpha use case: a military historian collaborating with archivists and museum curators to authenticate and share documentation of a WWII artifact.

### Actors

1. **Colonel Robert Johnson** (US Military) - Has access to classified artifact provenance
2. **Emily Davis** (Museum Curator) - Needs documentation for exhibit
3. **James Martinez** (National Archives) - Provides historical authentication
4. **You** (User) - Contact manager coordinating the collaboration

### Prerequisites

```bash
cd alpha
cargo build --release
mkdir -p data
cp config/credentials.toml.example config/credentials.toml
```

---

## Step 1: Import Initial Contacts

Import contacts from various sources:

```bash
# Import CSV with military and civilian contacts
./target/release/sagenscontact import --csv sample_data/contacts.csv

# Import vCard with social handles
./target/release/sagenscontact import --vcard sample_data/contacts.vcf

# Import SMS conversation history (mock)
./target/release/sagenscontact import --sms sample_data/sms_export.json
```

**Expected Output:**
```
Imported contact: John Doe
Imported contact: Jane Smith
Imported contact: Robert Johnson
Imported contact: Emily Davis
Imported contact: Michael Brown
Imported vCard content (mock parsing): 523 bytes
Imported SMS data (mock parsing)
Import completed successfully
```

---

## Step 2: List and Search Contacts

```bash
# List all contacts
./target/release/sagenscontact list

# Search for military contacts
./target/release/sagenscontact search "military"

# Search for museum curator
./target/release/sagenscontact search "Emily"
```

**Expected Output:**
```
Contacts:
  John Doe - john.doe@example.com - +1-555-0100
  Jane Smith - jane.smith@techcorp.com - +1-555-0101
  Robert Johnson - robert.j@military.mil - +1-555-0102
  Emily Davis - emily.davis@museum.org - +1-555-0103
  Michael Brown - mbrown@contractor.com - +1-555-0104

Found 1 contacts:
  Robert Johnson - robert.j@military.mil

Found 1 contacts:
  Emily Davis - emily.davis@museum.org
```

---

## Step 3: Create Project for Artifact Authentication

(Note: In alpha, project creation via CLI requires using the contact_id. First, get Robert's ID.)

```bash
# Add a note to Colonel Johnson about the artifact
./target/release/sagenscontact note <robert_contact_id> \
  "WWII Artifact - P-51 Mustang Pilot Logbook" \
  "Original flight logbook from Major Thomas Henderson, 332nd Fighter Group (Tuskegee Airmen). Serial #THH-1944-08. Requires authentication for museum exhibit. Contains 47 mission logs from 1944-1945 European theater."
```

**Expected Output:**
```
Note created: <note_uuid>
```

---

## Step 4: Get AI Suggestions

Request AI suggestions for organizing the contact:

```bash
./target/release/sagenscontact suggest <robert_contact_id>
```

**Expected Output:**
```
AI Suggestions for Robert Johnson:
  [0.85] Consider adding tags to better organize this contact. Based on their organization and title, they might fit into 'Business Partners' or 'Technical Leads' categories.
```

---

## Step 5: Share Artifact Documentation with Curator

Create a share invite to Emily Davis:

```bash
./target/release/sagenscontact share note <note_id> emily.davis@museum.org
```

**Expected Output:**
```
Share invite created: <invite_uuid>
```

---

## Step 6: Queue Communication for Follow-up

Send a follow-up reminder to Colonel Johnson:

```bash
./target/release/sagenscontact communicate <robert_contact_id> email \
  "Hi Colonel Johnson, just following up on the P-51 logbook authentication. Emily Davis from the Historical Society is ready to proceed with the exhibit. Can we schedule a call this week to discuss next steps?"
```

**Expected Output:**
```
[MOCK] Sending email to contact <robert_contact_id>
[MOCK] Subject: None
[MOCK] Message: Hi Colonel Johnson, just following up...
[MOCK] Email sent successfully (deterministic mock)
Communication queued: <attempt_uuid>
```

---

## Step 7: Queue Communication for Archivist

Reach out to James Martinez via LinkedIn:

```bash
./target/release/sagenscontact communicate <james_contact_id> linkedin \
  "James, we're working on authenticating a Tuskegee Airmen artifact. Would you be available to review the provenance documentation and provide a professional opinion?"
```

**Expected Output:**
```
[MOCK] Sending linkedin message to contact <james_contact_id>
[MOCK] Message: James, we're working on authenticating...
[MOCK] linkedin message sent successfully (deterministic mock)
Communication queued: <attempt_uuid>
```

---

## Step 8: Add Reminder Note

Create a note with next steps:

```bash
./target/release/sagenscontact note <robert_contact_id> \
  "Follow-up Actions" \
  "TODO:
  1. Wait for James Martinez authentication (ETA: 2 weeks)
  2. Schedule video call with Colonel Johnson and Emily Davis
  3. Arrange secure transfer of logbook to museum
  4. Draft exhibit description with Emily's input
  5. Set up communication nag reminder for 1 week out"
```

---

## Step 9: Run Sync Service (Optional)

Start the sync service to enable web/desktop access:

```bash
cargo run --bin sync_service
```

In another terminal, test the API:

```bash
# Health check
curl http://localhost:3000/health

# List contacts via API
curl http://localhost:3000/api/contacts

# Get AI suggestions
curl http://localhost:3000/api/ai/suggestions/<robert_contact_id>
```

---

## Step 10: Verify Data Persistence

Restart the CLI and verify contacts are still present:

```bash
./target/release/sagenscontact list
./target/release/sagenscontact search "Robert"
```

All contacts, notes, and communication attempts should persist.

---

## Alternative Workflow: SMS Import and Follow-up

### Import SMS Conversations

```bash
./target/release/sagenscontact import --sms sample_data/sms_export.json
```

This imports mock SMS conversations between you and Emily Davis, and you and Colonel Johnson, showing the progression of the artifact discussion.

### Queue SMS Reminder (CLI)

```bash
./target/release/sagenscontact communicate <emily_contact_id> sms \
  "Hi Emily, just checking in on the exhibit timeline. Are we still on track for the June opening?"
```

**Expected Output:**
```
✅ Communication queued: <attempt_uuid>

⚠️  [MOCK] This is a SIMULATED communication - NO actual SMS will be sent!

📋 Details:
   Recipient: Emily Davis
   Phone: +1-555-0103
   Message: Hi Emily, just checking in on the exhibit timeline...

💡 Alpha Limitation:
   All Email/SMS/Social sends are MOCKED in this release.
   The communication has been logged to the database but will NOT be
   delivered to any real service. This allows testing the workflow
   without requiring actual SMTP/SMS credentials.

🌐 Try the Web UI:
   Visit http://localhost:3001/communications to use the placeholder
   communication forms with explicit Email/SMS cards.

📊 To view queued communications:
   Check the database or run background worker to process mocks.
```

### Use Web UI for Communications (Alternative)

Instead of CLI, use the web interface:

```bash
# Start sync service
cargo run --release --bin sync_service &

# Start web UI
cd apps/web
pnpm install
pnpm dev

# Visit http://localhost:3001/communications
```

**Web UI Features:**
- Select contact from dropdown
- Choose Email or SMS tab
- Fill in recipient, subject/message
- See explicit "MOCK" warnings
- Get detailed feedback on submission
- All communication attempts logged to database

---

## Acceptance Criteria Verification

After completing this workflow, verify:

- [x] Desktop (CLI) build functional ✅
- [x] Web UI with placeholder communications ✅
- [x] One successful import flow (CSV, vCard, SMS) ✅
- [x] Contact edit (via note creation) ✅
- [x] Note attachment (simulated with note content) ✅
- [x] AI suggestion (mock) ✅
- [x] Communication attempt log ✅
- [x] **Email/SMS placeholder forms working** ✅
- [x] **CLI mock feedback clear** ✅
- [x] Sharing invite ✅
- [x] Offline change sync tested (data persistence) ✅
- [x] Placeholder credentials documented ✅

---

## Future Workflow (Beta)

In beta, this workflow will extend to:

1. **Real-time Collaboration**: Emily accepts share invite, views artifact note in web UI
2. **Desktop App**: View contacts, notes, and attachments in Tauri GUI
3. **Mobile Access**: Access shared artifact from phone via responsive web interface
4. **Actual Notifications**: Colonel Johnson receives email, Emily gets push notification
5. **Conflict Resolution**: Offline edits on desktop sync with web changes
6. **Secure Sharing**: End-to-end encryption for artifact documentation
7. **Virus Scanning**: Attachment uploads scanned before storage

---

## Troubleshooting

### "No such file or directory: data/contacts.db"
```bash
mkdir -p data
# Database will be created automatically on first run
```

### "Invalid UUID"
Use the actual UUID from the `list` or `note created` output, not placeholder text.

### Mock services not logging
Ensure `RUST_LOG=info` is set:
```bash
RUST_LOG=info ./target/release/sagenscontact communicate ...
```

### Sync service won't start
Check port 3000 is not in use:
```bash
lsof -i :3000
# Kill conflicting process if needed
```