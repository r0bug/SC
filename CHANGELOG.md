# Changelog

All notable changes to SagensContact will be documented in this file.

## [Unreleased - Beta]

### Planned
- Desktop app (Tauri + SvelteKit)
- Web app (SvelteKit SSR)
- JWT authentication
- Secure credential vault integration
- Real Segmind AI integration
- TLS for sync service
- End-to-end encryption
- Conflict resolution for sync
- Playwright E2E tests

## [0.1.0-alpha.1] - 2024-01-XX

### Alpha Status

This is an **ALPHA** release demonstrating core functionality with:
- ✅ CLI with CSV import (field mapping + preview)
- ✅ Web UI with contacts, notes, communications screens
- ✅ Background worker for queue processing
- ✅ Live API integration with database
- ⚠️ Mock external services (Email/SMS/AI)
- ⚠️ No authentication or encryption
- 📋 vCard/SMS import parsing pending
- 📋 Desktop app architecture documented, implementation pending
- 📋 Advanced web features (projects detail, sharing UI) pending

### Critical Fixes

#### Fixed - SQLx Migrations Blocker
- Replaced `sqlx::migrate!()` macro with direct schema execution
- Database now initializes correctly on first connection
- Resolves compile-time error requiring migrations/ directory

#### Fixed - Communication Queue Status Filtering
- Changed JSON status query from `json_extract()` to `LIKE` pattern
- Added retry count persistence to ensure failures eventually fail out
- Communication queue now correctly fetches pending/retrying attempts

#### Fixed - Sync Service API Integration
- Wired all API endpoints to real repositories
- Added `AppState` for shared database and AI client access
- Endpoints now perform actual CRUD operations instead of returning stubs
- Enabled SQLite foreign key enforcement for referential integrity

### Added

#### Core Features
- Core domain entities (Contact, Tag, Project, Note, Calendar, etc.)
- SQLite local storage with foreign key enforcement
- CLI client with CSV import (field mapping + preview), CRUD, search, communicate
- Sync service with REST API and WebSocket endpoints (port 3000)
- Web UI with contact list, detail, notes, communications (port 3001)
- Background worker binary for queue processing (30s intervals)
- Type-safe API client library for web UI
- Mock communication adapters (Email, SMS, Social)
- Mock AI middleware (Segmind)
- Communication queue with retry logic and persistence
- Nag scheduler with cron support

#### Sample Data
- CSV with 5 sample contacts
- vCard with 2 contacts including social handles
- SMS conversation export JSON
- Military artifact sharing demo scenario

#### Documentation
- README.md - Project overview with accurate alpha status
- QUICKSTART.md - Step-by-step setup guide with port configuration
- ARCHITECTURE.md - System design and data flow
- SECURITY_NOTES.md - Alpha limitations and threat model
- TESTING.md - Test strategy and CI/CD
- WORKFLOW_DEMOS.md - Complete demo walkthrough
- CHANGELOG.md (this file) - Current development status

#### Infrastructure
- GitHub Actions CI/CD pipeline
- E2E test script for CLI
- Rust toolchain configuration
- Code formatting standards
- Comprehensive .gitignore

#### Roadmap Documentation
- Desktop app architecture (apps/desktop/README.md)
- Web app architecture (apps/web/README.md)

### Changed
- Improved error handling in repositories
- Enhanced logging throughout application

### Known Limitations (Alpha)
- **Desktop app:** Architecture documented, implementation planned for beta
- **Web UI:** Core features implemented (contacts, notes, communications), advanced features pending
- **Import:** CSV with field mapping implemented, vCard/SMS parsing pending
- **Security:** No authentication, no encryption, plaintext credentials
- **External services:** All Email/SMS/AI interactions mocked
- **Sync:** Basic WebSocket support, no conflict resolution
- **Single-user:** No multi-tenancy or user isolation

## Development Notes

### Versioning Strategy
- Alpha: `0.1.0-alpha.x`
- Beta: `0.1.0-beta.x`
- Release Candidates: `0.1.0-rc.x`
- Stable: `0.1.0`

### Release Process
1. Update CHANGELOG.md
2. Bump version in Cargo.toml workspace
3. Run full test suite
4. Build release artifacts
5. Tag release in git
6. Publish release notes

### Breaking Changes Policy
- Alpha: Breaking changes allowed
- Beta: Breaking changes must be documented
- Stable: Follow semantic versioning strictly