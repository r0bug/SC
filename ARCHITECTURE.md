# Architecture

## Overview

SagensContact is a portable, multi-platform contact management system with offline-first capabilities and optional cloud sync.

## System Components

### Core Domain (`core_domain`)
Pure Rust domain models with no external dependencies:
- Entities: Contact, Tag, Project, CalendarEvent, Note, Attachment
- Communication: CommunicationAttempt, CommunicationMethod, CommunicationStatus
- Sharing: ShareInvite, ShareEntityType, Permission
- AI: AiInteraction (logging with feedback/cache/retry tracking)
- Search: SearchHistory (enriched with result_ids, privacy_mode, metadata)
- Attachment: Attachment with ScanStatus, checksum verification, encryption flag
- Email: EmailMessage, ImapAccount, EmailTriageSession
- SMS: SmsMessage, SmsConversation
- Concept Graph: Concept, CommunicationConcept, ConceptMatcherGroup, ConceptMatcher
- Detection: DetectionMethod (Keyword, Ai, Manual, Rule), MatcherElementType, MatchMode
- Contact Intelligence: Relationship, Location, Pick

**Design Principles:**
- Domain-driven design
- Platform-agnostic
- Serializable for network transport
- Extensible metadata JSON fields for future enhancement

### Local Store (`local_store`)
SQLite/PostgreSQL persistence layer with feature flags:
- Schema migrations via sqlx (embedded)
- Repository pattern for each entity type
- Transaction support
- Full-text search on contacts
- Database abstraction for SQLite (default) or PostgreSQL

**Data Flow:**
```
Application → Repository → SQLx → SQLite/PostgreSQL
```

**Feature Flags:**
- `sqlite` (default) - SQLite backend
- `postgres` - PostgreSQL backend

### Sync Service (`sync_service`)
Axum-based REST API with WebSocket support:
- HTTP endpoints for CRUD operations
- WebSocket for real-time notifications
- CORS-enabled for web clients
- Structured logging with tracing
- ACL enforcement on all routes
- Comprehensive audit logging

**Endpoints:**
- `GET /health` - Health check
- `GET/POST /api/contacts` - Contact CRUD
- `GET/POST /api/tags` - Tag management
- `GET/POST /api/projects` - Project management
- `POST /api/notes` - Note creation
- `POST /api/communication` - Queue communication
- `POST /api/share` - Create share invite
- `GET /api/ai/suggestions/:id` - AI suggestions
- `POST /api/attachments` - File upload with virus scanning
- `GET /api/import/preview` - Import preview
- `POST /api/import` - Execute import
- `GET /ws` - WebSocket upgrade
- **Email:** `/api/imap-accounts`, `/api/emails`, `/api/email-import`
- **SMS:** `/api/sms`, `/api/android-import`
- **Email Triage:** `/api/email-triage`
- **Concepts:** `/api/concepts`, `/api/concepts/:id/matchers`, `/api/concepts/:id/scan`
- **Labels:** `/api/labels`, `/api/labels/:id`, `/api/labels/scan-all`
- **Communication Concepts:** `/api/communication-concepts`
- **Matches:** `/api/matches`
- **Manual Domains:** `/api/manual-domains`
- **Reconciliation:** `/api/reconciliation`
- **Relationships:** `/api/relationships`
- **Locations:** `/api/locations`
- **Picks:** `/api/picks`

### Communication Queue (`communication_queue`)
Background job processing for outbound communications:
- Email adapter (SMTP) - configurable via environment variables
- SMS adapter (Twilio) - configurable via environment variables
- Social adapters (mock in alpha, OAuth2 planned)
- Retry logic with exponential backoff
- Cron-based scheduling for nag reminders
- Status tracking in database

**Flow:**
```
CLI/UI → Queue Communication → Database
       ↓
Background Worker → Process Pending → Adapter → External Service
                                    ↓
                                Update Status
```

**Configuration:**
- `SMTP_HOST`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM` - Real email
- `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_PHONE_NUMBER` - Real SMS
- Without config, adapters log to console (fallback mode)

### AI Middleware (`ai_middleware`)
Configurable AI integration with logging and caching:

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
AiInteractionRepository → SQLite/PostgreSQL
```

**Tracked Metadata:**
- Prompt and response text
- Confidence score and model name
- Entity context (type/id)
- Cache hit status (true/false, cached_at timestamp)
- Retry attempts (count, backoff_ms array)
- User feedback (helpful boolean, applied boolean, timestamps)

**Configuration:**
- Set `SEGMIND_API_KEY` for real AI suggestions
- Without API key, returns deterministic mock responses

### Import Service (`import_service`)
Multi-format contact import with preview:
- CSV with field mapping
- vCard (.vcf)
- Social media exports (LinkedIn, Twitter/X, Facebook, Instagram)
- Android SMS/call history
- Smart detection of format

### Attachment Service (`attachment_service`)
File storage with virus scanning:
- Local filesystem storage (default)
- S3-compatible storage (MinIO, AWS S3, DigitalOcean Spaces)
- ClamAV virus scanning integration
- SHA-256 checksum verification
- Thumbnail generation for images

**Storage Backends:**
- `LocalStorage` - filesystem based
- `S3Storage` - S3-compatible (requires `s3-storage` feature)

### Cache Layer (`cache_layer`)
Caching infrastructure:
- In-memory cache using moka (default)
- Redis backend (optional, for distributed caching)
- Specialized caches: AI responses, sessions, rate limiting

**Feature Flags:**
- `memory` (default) - moka in-memory cache
- `redis` - Redis backend

### Secure Vault (`secure_vault`)
Encrypted credential storage:
- AES-256-GCM encryption
- Environment variable injection at runtime
- CLI tool for encrypting credential files

### CLI Client (`cli_client`)
Command-line interface with clap:
- Import: CSV, vCard, JSON (social), SMS
- CRUD operations on contacts
- Search functionality
- Note creation and attachment
- Communication queuing
- Share invite creation
- AI suggestion requests

## Attachment Pipeline Architecture

### Upload Flow
```
Web/Desktop Client
    ↓ (multipart/form-data)
Sync Service Endpoint (/api/attachments)
    ↓
Calculate SHA-256 Checksum
    ↓
Save to Storage (Local or S3)
    ↓
Create Attachment Record (status: Pending)
    ↓
Virus Scanner (ClamAV or mock)
    ↓
Update scan_status → Clean/Infected/Error
    ↓
If Infected: Delete file, mark attachment
    ↓
Return Attachment Metadata to Client
```

### Download Flow
```
Web/Desktop Client
    ↓ (GET /api/attachments/{id}/download)
Sync Service Endpoint
    ↓
Verify scan_status == Clean
    ↓
Read file from storage
    ↓
Calculate checksum, compare to stored
    ↓
If match: Stream file to client
If mismatch: Return error (file corruption)
```

## AI Suggestion Flow

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
    ↓ (real API or mock)
Segmind API / Mock Response
    ↓
Store in Cache
    ↓
Log to AiInteractionRepository
    ↓
Return Suggestion to Client
```

## Data Flow Diagrams

### Import Workflow
```
CSV/vCard/JSON File
    ↓
Import Service Parser
    ↓
Preview (field mapping, sample records)
    ↓
User Confirmation
    ↓
Domain Entity Creation
    ↓
Repository.create()
    ↓
SQLite/PostgreSQL Database
```

### Sync Workflow
```
Local SQLite ←→ Sync Service ←→ PostgreSQL
                     ↓
                 WebSocket
                     ↓
            Desktop/Web Clients
```

### Communication Flow
```
User → CLI/Web communicate command
    ↓
CommunicationAttempt created (status: Pending)
    ↓
Saved to database
    ↓
Background worker polls for pending
    ↓
Adapter processes attempt (SMTP/Twilio/mock)
    ↓
Update status (Sent/Failed/Retrying)
```

## Database Schema

Key tables:
- `contacts` - Core contact information with full-text search
- `social_handles` - Social media profiles (1:N with contacts)
- `tags` - User-defined tags
- `contact_tags` - Many-to-many relationship
- `projects` - Project grouping
- `project_contacts` - Many-to-many relationship
- `notes` - Polymorphic notes (Contact, Project, CalendarEvent)
- `communication_attempts` - Outbound communication log
- `share_invites` - Sharing permissions
- `resource_acls` - Fine-grained access control
- `audit_logs` - Security audit trail

**Attachment Tables:**
- `attachments` - Polymorphic file attachments
  - Fields: checksum (SHA-256), scan_status, encrypted, storage_path
  - Supports: Contact, Project, Note, CalendarEvent, Communication

**AI Tables:**
- `ai_interactions` - Logged AI suggestion activity
  - Fields: prompt, response, confidence, model, entity_type, entity_id
  - Metadata: cache_hit, retries, backoff_ms
  - Feedback: helpful, applied, feedback_at

**Search Tables:**
- `search_history` - Enriched search tracking
  - Fields: query, filters, result_count, result_ids
  - Privacy: privacy_mode (boolean)

**Email Tables:**
- `imap_accounts` - IMAP account credentials and sync state
- `email_messages` - Full email content with headers, body, attachments
- `email_triage_sessions` - AI-powered domain discovery sessions

**SMS Tables:**
- `sms_messages` - SMS message content and metadata
- `sms_conversations` - SMS conversation threads

**Concept Graph Tables:**
- `concepts` - Named concepts/domains for categorization
- `communication_concepts` - Links between communications and concepts (with status, confidence, detection_method)
- `concept_matcher_groups` - Matcher criteria groups (OR'd together in DNF)
- `concept_matchers` - Individual matchers within groups (AND'd together)
  - element_type: sender, receiver, subject, body, attachment_name, any_text
  - match_mode: contains, exact, starts_with, ends_with
  - Boolean: negate flag for NOT logic

**Contact Intelligence Tables:**
- `relationships` - Contact-to-contact relationships with types
- `locations` - Contact locations/addresses
- `picks` - User selections on AI-generated suggestions

## Security Architecture

**Implemented:**
- ACL enforcement on all API routes
- Audit logging for security events
- Password validation (strength requirements)
- JWT session management
- CORS configuration
- File upload validation (type, size, extension)
- Virus scanning (ClamAV integration)
- Encrypted credential vault

**Configuration Required:**
- TLS/HTTPS (reverse proxy recommended)
- SQLCipher for database encryption (planned)
- OAuth2 for social platforms (planned)

## Deployment Models

### Development
- SQLite database
- In-memory cache
- Local file storage
- Mock external services (no config required)

### Production
- PostgreSQL database
- Redis for distributed caching
- S3-compatible storage for attachments
- ClamAV for virus scanning
- Real SMTP/Twilio for communications
- TLS via reverse proxy (nginx/Caddy)

## Technology Stack

**Backend:**
- Rust 1.83+
- Tokio async runtime
- Axum web framework
- SQLx for database access (SQLite/PostgreSQL)
- Serde for serialization
- Tower for middleware

**Frontend:**
- Tauri for desktop shell
- SvelteKit for UI
- TypeScript
- Tailwind CSS

**Infrastructure:**
- SQLite (local) / PostgreSQL (production)
- Redis (optional, for caching)
- MinIO / S3 (optional, for attachments)
- ClamAV (optional, for virus scanning)

## Testing Strategy

See [TESTING.md](TESTING.md) for details.

- Unit tests: Core domain logic and repository functions
- Integration tests: Database operations and API endpoints
- E2E tests: CLI script and Playwright web tests
