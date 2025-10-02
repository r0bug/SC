# Architecture

## Overview

SagensContact is a portable, multi-platform contact management system with offline-first capabilities and optional cloud sync.

## System Components

### Core Domain (`core_domain`)
Pure Rust domain models with no external dependencies:
- Entities: Contact, Tag, Project, CalendarEvent, Note, Attachment
- Communication: CommunicationAttempt, CommunicationMethod, CommunicationStatus
- Sharing: ShareInvite, ShareEntityType, Permission
- AI: AiInteraction (Phase 6: logging with feedback/cache/retry tracking)
- Search: SearchHistory (Phase 6: enriched with result_ids, privacy_mode, metadata)
- Attachment: Attachment with ScanStatus, checksum verification, encryption flag

**Design Principles:**
- Domain-driven design
- Platform-agnostic
- Serializable for network transport
- Extensible metadata JSON fields for future enhancement

### Local Store (`local_store`)
SQLite-based persistence layer:
- Schema migrations via sqlx
- Repository pattern for each entity type
- Transaction support
- Full-text search on contacts

**Data Flow:**
```
Application → Repository → SQLx → SQLite
```

### Sync Service (`sync_service`)
Axum-based REST API with WebSocket support:
- HTTP endpoints for CRUD operations
- WebSocket for real-time notifications
- CORS-enabled for web clients
- Structured logging with tracing

**Endpoints:**
- `GET /health` - Health check
- `GET/POST /api/contacts` - Contact CRUD
- `GET/POST /api/tags` - Tag management
- `GET/POST /api/projects` - Project management
- `POST /api/notes` - Note creation
- `POST /api/communication` - Queue communication
- `POST /api/share` - Create share invite
- `GET /api/ai/suggestions/:id` - AI suggestions
- `GET /ws` - WebSocket upgrade

### Communication Queue (`communication_queue`)
Background job processing for outbound communications:
- Mock adapters for Email, SMS, Social platforms
- Retry logic with exponential backoff
- Cron-based scheduling for nag reminders
- Status tracking in database

**Flow:**
```
CLI/UI → Queue Communication → Database
       ↓
Background Worker → Process Pending → Adapter → [MOCK] External Service
                                    ↓
                                Update Status
```

### AI Middleware (`ai_middleware`)
Configurable AI integration with logging and caching (Phase 6):

**Components:**
- `SegmindClient` - API client with retry logic (3 attempts, exponential backoff)
- `LoggingSegmindClient` - Wrapper that logs all interactions to database
- Response cache - 1-hour TTL to reduce API costs

**Logging Architecture:**
```
Application
    ↓
LoggingSegmindClient (wrapper)
    ↓ (delegates to)
SegmindClient (with cache)
    ↓ (mock or real API)
Segmind API / Mock
    ↑
LoggingSegmindClient
    ↓ (logs to)
AiInteractionRepository → SQLite
```

**Tracked Metadata:**
- Prompt and response text
- Confidence score and model name
- Entity context (type/id)
- Cache hit status (true/false, cached_at timestamp)
- Retry attempts (count, backoff_ms array)
- User feedback (helpful boolean, applied boolean, timestamps)

**Alpha vs Production:**
- **Alpha**: Mock mode with deterministic responses (no API key required)
- **Beta**: Real Segmind API calls with environment variable `SEGMIND_API_KEY`
- **Cache**: TTL-based in-memory cache (upgradeable to Redis in production)

### CLI Client (`cli_client`)
Command-line interface with clap:
- Import: CSV, vCard, SMS JSON
- CRUD operations on contacts
- Search functionality
- Note creation and attachment
- Communication queuing
- Share invite creation
- AI suggestion requests

## Phase 6: Attachment & AI Pipeline Architecture

### Attachment Upload Flow
```
Web/Desktop Client
    ↓ (multipart/form-data)
Sync Service Endpoint (/api/attachments)
    ↓
Calculate SHA-256 Checksum
    ↓
Save to Local Storage (./data/attachments/{entity_type}/{uuid})
    ↓
Create Attachment Record (status: Pending)
    ↓
AttachmentRepository.create() → SQLite
    ↓
[Background] Virus Scanner (Mock: always Clean)
    ↓
Update scan_status → Clean/Infected/Error
    ↓
If Infected: Delete file, mark attachment
    ↓
Return Attachment Metadata to Client
```

### Attachment Download Flow
```
Web/Desktop Client
    ↓ (GET /api/attachments/{id}/download)
Sync Service Endpoint
    ↓
AttachmentRepository.get_by_id()
    ↓
Verify scan_status == Clean
    ↓
Read file from storage_path
    ↓
Calculate checksum, compare to stored
    ↓
If match: Stream file to client
If mismatch: Return error (file corruption)
```

### AI Suggestion Flow with Logging
```
Web/Desktop Client
    ↓ (POST /api/ai/suggest)
Sync Service Handler
    ↓
LoggingSegmindClient.generate_suggestion_with_logging()
    ↓
Check Cache (1-hour TTL)
    ↓ (cache miss)
SegmindClient (with retry logic: 3x, exponential backoff)
    ↓ (mock or real API)
Segmind API / Mock Response
    ↓
Store in Cache
    ↓
Log to AiInteractionRepository
  - prompt, response, confidence, model
  - metadata: {cache_hit: false, retries: 0}
  - entity_type, entity_id for context
    ↓
Return Suggestion to Client
```

### AI Feedback Loop
```
User provides feedback (helpful/applied)
    ↓ (POST /api/ai/feedback/{interaction_id})
Sync Service Handler
    ↓
AiInteractionRepository.update_feedback()
  - feedback_helpful = true/false
  - feedback_applied = true (if user clicked "Apply")
  - feedback_at = NOW()
    ↓
Analytics: Track suggestion quality over time
```

### Search History Enrichment
```
User performs search
    ↓
SearchHistoryRepository.create() (if privacy_mode=false)
  - query: "john doe"
  - filters: {tags: ["military"], organization: "US Army"}
  - result_count: 5
  - result_ids: [uuid1, uuid2, uuid3, uuid4, uuid5]
  - privacy_mode: false
  - metadata: {search_duration_ms: 42}
    ↓
Recent Searches Widget queries last 10
    ↓
Filter out privacy_mode=true entries
    ↓
Display with result_count and timestamps
```

## Data Flow Diagrams

### Import Workflow
```
CSV/vCard/SMS File
    ↓
CLI Parser
    ↓
Domain Entity Creation
    ↓
Repository.create()
    ↓
SQLite Database
```

### Sync Workflow (Future)
```
Local SQLite ←→ Sync Service ←→ PostgreSQL
                     ↓
                 WebSocket
                     ↓
            Desktop/Web Clients
```

### Communication Flow
```
User → CLI communicate command
    ↓
CommunicationAttempt created (status: Pending)
    ↓
Saved to database
    ↓
Background job polls for pending
    ↓
Adapter processes attempt
    ↓
Update status (Sent/Failed/Retrying)
```

## Database Schema

Key tables:
- `contacts` - Core contact information
- `social_handles` - Social media profiles (1:N with contacts)
- `tags` - User-defined tags
- `contact_tags` - Many-to-many relationship
- `projects` - Project grouping
- `project_contacts` - Many-to-many relationship
- `notes` - Notes attached to contacts or projects
- `communication_attempts` - Outbound communication log
- `share_invites` - Sharing permissions

**Phase 6 Tables:**
- `attachments` - Polymorphic file attachments (Contact, Project, Note, CalendarEvent, Communication)
  - Fields: checksum (SHA-256), scan_status (Pending/Clean/Infected/Error), encrypted (boolean)
  - Storage: Local filesystem (alpha), S3-compatible (beta)
- `ai_interactions` - Logged AI suggestion activity
  - Fields: prompt, response, confidence, model, entity_type, entity_id
  - Metadata JSON: cache_hit, retries, backoff_ms
  - Feedback: helpful (boolean), applied (boolean), feedback_at (timestamp)
- `search_history` - Enriched search tracking
  - Fields: query, filters (JSON), result_count, result_ids (JSON array of UUIDs)
  - Privacy: privacy_mode (boolean) - when true, search not stored
  - Metadata JSON: extensible for future analytics

Indexes:
- `idx_contacts_email`
- `idx_contacts_phone`
- `idx_social_handles_contact`
- `idx_notes_contact`
- `idx_notes_project`
- `idx_comm_attempts_contact`
- `idx_comm_attempts_status`
- `idx_attachments_entity` (Phase 6: entity_type + entity_id)
- `idx_ai_interactions_user` (Phase 6: user_id + created_at)
- `idx_ai_interactions_entity` (Phase 6: entity_type + entity_id)
- `idx_search_history_user` (Phase 6: user_id + created_at)

## Security Considerations (Alpha)

**Current State:**
- No authentication or authorization
- Plaintext credential files
- No encryption at rest
- No TLS for sync service
- Mock external service calls

**Future Hardening:**
- Integrate with OS keychain for credential storage
- Add JWT-based authentication
- Implement RBAC for sharing
- Enable TLS for all network traffic
- Encrypt sensitive fields in database
- Add virus scanning for attachments
- Implement rate limiting on API endpoints

## Deployment Models

### Alpha (Current)
- Single-user desktop application
- Local SQLite database
- Optional sync service on localhost
- No external network dependencies

### Beta (Future)
- Multi-user with authentication
- Self-hosted sync service option
- Secure credential vault
- Mobile web interface

### Production (Future)
- Hosted SaaS option
- Postgres with connection pooling
- Redis for caching and job queue
- S3-compatible storage for attachments
- Horizontal scaling of sync service
- CDN for web assets

## Testing Strategy

See TESTING.md for details.

## Technology Stack

**Backend:**
- Rust 1.75+
- Tokio async runtime
- Axum web framework
- SQLx for database access
- Serde for serialization

**Frontend (Planned):**
- Tauri for desktop shell
- SvelteKit for UI
- TypeScript
- Tailwind CSS

**Infrastructure:**
- SQLite (local)
- PostgreSQL (sync service)
- MinIO (development S3)
- Redis (optional job queue)