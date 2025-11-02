# SagensContact - Client Feature Matrix

This document tracks all functionality across Desktop (Tauri), Web (SvelteKit), and CLI clients to ensure feature parity.

**Legend:**
- ✅ Fully implemented
- 🚧 Partially implemented
- ❌ Not implemented
- N/A - Not applicable for this client

---

## Contact Management

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| List contacts | ✅ | ✅ | ✅ | All clients support pagination |
| Filter contacts by name presence | ✅ | ✅ | ✅ | `--only-named` flag in CLI |
| Search contacts | ✅ | ✅ | ✅ | Full-text search with debouncing (300ms) in GUI |
| View contact details | ✅ | ✅ | ❌ | CLI shows summary only |
| Create contact | ✅ | ✅ | ✅ | |
| Update contact | ✅ | ✅ | ❌ | |
| Delete contact | ✅ | ✅ | ❌ | |
| Import contacts (CSV) | ✅ | ✅ | ✅ | |
| Import contacts (vCard) | 🚧 | 🚧 | ✅ | Backend ready, UI pending |
| Import SMS history | ✅ | ❌ | ✅ | Desktop has file picker dialog |
| Import Google Contacts CSV | ❌ | ❌ | ✅ | Large file streaming import |
| Smart import (AI field mapping) | 🚧 | 🚧 | ❌ | Backend ready, UI in progress |
| Export contacts | ❌ | ❌ | ❌ | Planned for beta |
| Contact deduplication | 🚧 | 🚧 | ✅ | API available, UI pending |
| Merge duplicate contacts | 🚧 | 🚧 | ✅ | API available, UI pending |

---

## Communication History

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| View all communications | ✅ | ✅ | ❌ | |
| View SMS threads | ✅ | ❌ | ❌ | Thread grouping with collapsible UI |
| View by communication type | ✅ | ❌ | ❌ | Tab interface (All/Threads) |
| Queue communication attempt | 🚧 | 🚧 | ✅ | |
| Filter by date range | ❌ | ❌ | ❌ | Planned |
| Filter by communication type | ❌ | ❌ | ❌ | Planned |
| Filter by last contact date | ❌ | ❌ | ❌ | User requested, pending |

---

## Groups

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| List groups | ✅ | ✅ | ✅ | |
| Create group | ✅ | ✅ | ✅ | |
| Update group | ✅ | ✅ | ❌ | |
| Delete group | ✅ | ✅ | ✅ | |
| Add member to group | ✅ | ✅ | ✅ | Recently added (2025-11) |
| Remove member from group | ✅ | ✅ | ✅ | Recently added (2025-11) |
| View group members | ✅ | ✅ | ❌ | |

---

## Projects

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| List projects | ✅ | ✅ | ❌ | |
| View project details | ✅ | ✅ | ❌ | |
| Create project | ✅ | ✅ | ❌ | |
| Update project | ✅ | ✅ | ❌ | |
| Delete project | ✅ | ✅ | ❌ | |
| Add contact to project | ✅ | ✅ | ❌ | |
| Remove contact from project | ✅ | ✅ | ❌ | |

---

## Calendar & Events

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| List events | ✅ | ✅ | ✅ | |
| View event details | ✅ | ✅ | ❌ | Event detail page with full information |
| Create event | ✅ | ✅ | ✅ | Date picker in GUI, /calendar/new route |
| Update event | ✅ | ✅ | ✅ | |
| Delete event | ✅ | ✅ | ✅ | |
| Filter by date range | ✅ | ✅ | ✅ | |
| Calendar view | ❌ | ❌ | N/A | Planned for beta |
| Event reminders | ❌ | ❌ | ❌ | Planned for beta |

---

## Notes

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| List notes | ✅ | ✅ | ❌ | |
| View note details | ✅ | ✅ | ❌ | |
| Create note | ✅ | ✅ | ✅ | Supports Contact, Project, Event entities |
| Update note | ❌ | ❌ | ❌ | Backend ready, UI pending |
| Delete note | ❌ | ❌ | ❌ | Backend ready, UI pending |
| Rich text editing | ❌ | ❌ | N/A | Planned for beta |

---

## Attachments

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| Upload attachment | ✅ | ✅ | ✅ | File picker in GUI |
| Download attachment | ✅ | ✅ | ✅ | |
| List attachments | ✅ | ✅ | ✅ | By entity type/ID |
| Delete attachment | ✅ | ✅ | ✅ | |
| Virus scanning | 🚧 | 🚧 | 🚧 | Mock implementation (alpha) |
| Cloud storage (S3) | ❌ | ❌ | ❌ | Planned for beta |
| File preview | ❌ | ❌ | N/A | Planned for beta |

---

## AI Features

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| Generate contact suggestions | 🚧 | 🚧 | ✅ | Segmind API integration |
| AI-powered field mapping (import) | 🚧 | 🚧 | ❌ | Smart import feature |
| Review AI insights | ❌ | ❌ | ✅ | |
| Apply AI insight | ❌ | ❌ | ✅ | |
| Provide feedback on insights | ❌ | ❌ | ✅ | Rating + comments |
| List AI insights | ❌ | ❌ | ✅ | |
| Mock mode (no API key) | ✅ | ✅ | ✅ | Falls back to mock responses |

---

## Sharing & Collaboration

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| Share entity by email | 🚧 | 🚧 | ✅ | |
| List share invites | ❌ | ❌ | ✅ | |
| Accept share invite | ❌ | ❌ | ✅ | |
| Revoke share | ❌ | ❌ | ✅ | |
| Update share permissions | ❌ | ❌ | ✅ | |

---

## Authentication & Sync

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| Local-first (SQLite) | ✅ | N/A | ✅ | Desktop & CLI use local DB |
| Login to sync service | ❌ | ✅ | ✅ | |
| Signup | ❌ | ✅ | ✅ | |
| Logout | ❌ | ✅ | ✅ | |
| Check auth status | ❌ | ❌ | ✅ | |
| Sync with server | ❌ | N/A | ✅ | |
| Offline support | ✅ | ❌ | ✅ | Desktop caches data |
| Check online status | ❌ | ❌ | ✅ | |

---

## Search & History

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| Full-text contact search | ✅ | ✅ | ✅ | SQLite FTS5 |
| Search history tracking | 🚧 | 🚧 | ✅ | Backend ready, UI pending |
| View search history | ❌ | ❌ | ✅ | |
| Clear search history | ❌ | ❌ | ✅ | |
| Privacy mode (no tracking) | 🚧 | 🚧 | 🚧 | Backend support exists |

---

## Settings & Configuration

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| View settings | ❌ | ❌ | ❌ | |
| Update settings | ❌ | ❌ | ❌ | |
| Configure API endpoints | ❌ | ❌ | ❌ | |
| Configure credentials | ❌ | ❌ | 🚧 | Via config/credentials.toml |
| Theme selection | ❌ | ❌ | N/A | Planned for beta |

---

## System Integration

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| System tray icon | ✅ | N/A | N/A | Linux/Windows, macOS behavior differs |
| Native notifications | ✅ | 🚧 | N/A | Tauri plugin |
| File system integration | ✅ | N/A | ✅ | Native file dialogs |
| System-wide shortcuts | ❌ | N/A | N/A | Planned for beta |
| Auto-start on login | ❌ | N/A | N/A | Planned for beta |

---

## Data Management

| Feature | Desktop | Web | CLI | Notes |
|---------|---------|-----|-----|-------|
| Database migrations | ✅ | N/A | ✅ | Automatic via sqlx |
| Data backup | ❌ | ❌ | ❌ | Planned |
| Data restore | ❌ | ❌ | ❌ | Planned |
| Open data directory | ✅ | N/A | N/A | Via system file browser |
| Database size stats | ❌ | ❌ | ❌ | Planned |

---

## Recent Updates (2025-11-02)

### Added:
1. **Contact Filtering** (All clients)
   - "Only show contacts with names" filter to hide number-only contacts
   - Desktop: Checkbox filter in contacts page
   - Web: Checkbox filter in contacts page
   - CLI: `--only-named` flag for list command

2. **Search Improvements** (Desktop & Web)
   - Fixed search reverting to all contacts issue
   - Implemented 300ms debouncing
   - Changed to client-side filtering for better performance
   - Eliminated race conditions

3. **SMS Thread View** (Desktop only)
   - Tabbed interface (All Communications / SMS Threads)
   - Thread grouping by thread_id
   - Chat-style message bubbles (gray=inbound, blue=outbound)
   - Collapsible thread items with message count

4. **Group Management** (All clients)
   - Added `add_group_member` command/API
   - Added `remove_group_member` command/API
   - Full CRUD operations now available

5. **Group Creation UI** (Desktop & Web)
   - Desktop: Separate pages for create (/groups/new) and manage (/groups/[id])
   - Desktop: Two-column layout for adding/removing members
   - Web: Modal-based create/edit with member selection
   - Both clients support searching contacts to add as members

6. **Calendar Event Pages** (Web App)
   - Added /calendar/new page for creating new events
   - Added /calendar/[id] page for viewing event details
   - Fixed broken links from calendar list page
   - Now has feature parity with Desktop app for calendar

---

## Priority Roadmap

### High Priority (Next Sprint)
- [ ] Contact deduplication UI (Desktop & Web)
- [ ] Contact merge UI (Desktop & Web)
- [ ] Update/delete operations for notes
- [ ] Filter communications by date range
- [ ] Filter contacts by last communication date

### Medium Priority
- [ ] Calendar view (month/week/day)
- [ ] Event reminders
- [ ] Settings/preferences UI
- [ ] Sharing UI (Desktop & Web)
- [ ] Search history UI (Desktop & Web)

### Low Priority
- [ ] Rich text note editing
- [ ] File preview for attachments
- [ ] Cloud storage integration (S3)
- [ ] System-wide shortcuts
- [ ] Auto-start configuration

---

## Testing Checklist

When implementing a new feature, verify across all applicable clients:

- [ ] Desktop app implements the feature with native UI
- [ ] Web app has equivalent functionality
- [ ] CLI has equivalent command (if applicable)
- [ ] Feature is documented in this file
- [ ] Backend API supports all required operations
- [ ] Error handling is consistent across clients
- [ ] Data validation is consistent
- [ ] User feedback (success/error messages) is clear

---

## Notes

- **Desktop app** is the primary development target (most features appear here first)
- **Web app** focuses on remote access and collaboration features
- **CLI** is designed for scripting, automation, and batch operations
- All clients share the same backend crates (local_store, core_domain, etc.)
- Desktop and CLI use local SQLite; Web uses sync service API

---

**Last Updated:** 2025-11-02
**Version:** Alpha v0.1.0-alpha.3+
