# SagensContact Code Quality & Best Practices Overhaul

## Introduction
Comprehensive cleanup of the SagensContact codebase to remove redundant code, fix warnings, improve type safety, and align with industry best practices. Based on a full audit of the Rust backend and SvelteKit frontend.

## Goals
- Zero clippy warnings
- Zero svelte-check warnings
- Remove all dead/redundant code
- Eliminate `any` types in TypeScript
- Fix all production `.unwrap()` calls
- Standardize error handling patterns
- Improve accessibility

## User Stories

### US-001: Remove dead frontend files (client.ts, enhanced-client.ts, websocket services, duplicate components)
Delete unused API clients, WebSocket services, and duplicate components that are never imported.
- [x] Delete `apps/web/src/lib/api/client.ts` (172 lines, never imported by pages)
- [x] Delete `apps/web/src/lib/api/enhanced-client.ts` (222 lines, never imported)
- [x] Delete `apps/web/src/lib/services/websocket.ts` (97 lines, unused)
- [x] Delete `apps/web/src/lib/services/resilient-websocket.ts` (298 lines, unused)
- [x] Delete `apps/web/src/lib/components/ToastContainer.svelte` (136 lines, ui/Toast.svelte is used instead)
- [x] Delete `apps/web/src/lib/components/LoadingSpinner.svelte` (68 lines, ui/LoadingSpinner.svelte is used)
- [x] Verify `npx svelte-check` still passes with 0 errors after deletions

### US-002: Fix all cargo clippy warnings in import_service (37 warnings)
Address the 37 clippy warnings in the import_service crate.
- [x] Replace `&Box<dyn ImportConnector>` with `&dyn ImportConnector` (3 instances)
- [x] Add `Default` impl for types with `new()` (8 instances)
- [x] Replace `.len() == 0` with `.is_empty()` (5 instances)
- [x] Replace `get(0)` with `first()` (2 instances)
- [x] Fix redundant closures, manual flatten, needless range loops, and other minor lints
- [x] `cargo clippy -p import_service -- -W clippy::all` returns 0 warnings

### US-003: Fix all cargo clippy warnings in sync_service (10 warnings)
Address the 10 clippy warnings in the sync_service crate.
- [x] Refactor `AppState::new()` to take a config struct instead of 13 arguments
- [x] Refactor `log_operation` to take a struct instead of 8 arguments
- [x] Refactor `fetch_emails` to take a config/params struct instead of 8 arguments
- [x] Replace `match ... { Ok(_) => true, Err(_) => false }` with `.is_ok()` (1 instance)
- [x] Replace `map_or(false, ...)` with `is_some_and()` (4 instances)
- [x] Remove unnecessary `u32` to `u32` cast (1 instance)
- [x] Replace manual char comparison with char array (1 instance)
- [x] `cargo clippy -p sync_service -- -W clippy::all` returns 0 warnings

### US-004: Fix clippy warnings in remaining crates (cache_layer, ai_middleware, local_store, cli_client)
Address the 16 remaining clippy warnings across 4 crates.
- [x] cache_layer: Remove unneeded `return` statement (line 197)
- [x] ai_middleware: Refactor `log_interaction` to take a struct instead of 9 arguments
- [x] local_store: Replace 4 redundant closures and 1 `or_insert_with(Vec::new)` with `or_default()`
- [x] cli_client: Replace `or_insert_with(Vec::new)` with `or_default()`
- [x] Fix 6 compilation warnings (2 unused imports in test files, 3 unnecessary `mut`, 1 useless comparison)
- [x] `cargo clippy -- -W clippy::all` returns 0 warnings across entire workspace

### US-005: Fix production .unwrap() calls in Rust backend
Replace `.unwrap()` in non-test code with proper error handling.
- [x] `crates/local_store/src/repositories/contact.rs:33,387` — Replaced `.unwrap()` with `.unwrap_or_default()`
- [x] `crates/sync_service/src/websocket.rs:137` — Replaced `.unwrap()` with match + error log + early return
- [x] `crates/sync_service/src/import_routes.rs:153` — Replaced `.unwrap()` with `.unwrap_or("unknown")`
- [x] `cargo test` still passes (198 tests — count reduced due to dead test removal)

### US-006: Remove dead code markers in Rust backend
Remove or properly use the 28+ items marked `#[allow(dead_code)]`.
- [x] Review and remove unused structs/fields in `sync_service/src/ws.rs` (if superseded by websocket.rs)
- [x] Review and remove unused validation constants in `sync_service/src/validation.rs` (lines 9-23)
- [x] Review and remove unused functions in `sync_service/src/update_system.rs` — kept with `#[allow(dead_code)]` (future update workflow)
- [x] Review and remove unused functions in `sync_service/src/observability.rs` — kept with `#[allow(dead_code)]` (metrics infrastructure)
- [x] Review and remove unused functions in `sync_service/src/audit.rs`
- [x] Review and remove unused functions in `cli_client/src/import.rs` — removed 180 lines of dead import functions
- [x] `cargo check` passes with no new warnings

### US-007: Replace `any` types with proper TypeScript types in frontend
Eliminate the 45+ `any` usages across the frontend codebase.
- [x] Type `email-domains/+page.svelte` domains array with proper interface
- [x] Type `email-explorer/+page.svelte` domains, emails, overlaps, selectedEmail with proper interfaces
- [x] Type `locations/+page.svelte` data objects
- [x] Type `dashboard/+page.svelte` updateInfo and status functions
- [x] Replace `catch (error: any)` with `catch (error: unknown)` and type guards across all files
- [x] Type WebSocket callback parameters in api.ts
- [x] Type optimistic store data fields
- [x] `npx svelte-check` returns 0 errors — only 1 intentional `any` remaining (private `request()` method)

### US-008: Fix accessibility warnings in Svelte components (92 warnings)
Add proper ARIA attributes and roles to interactive elements.
- [x] Add `role="button"` and `tabindex="0"` to clickable `<div>` elements (22 instances)
- [x] Add `role="option"` to clickable `<li>` elements (6 instances)
- [x] Add `<label>` elements or `aria-label` to form inputs (27 instances)
- [x] Remove unused CSS selectors (concepts/+page.svelte:315, labels/[id]:501, locations:205)
- [x] Remove unused export props from ElementSelector.svelte (subject, bodyText)
- [x] `npx svelte-check` returns 0 warnings

### US-009: Extract shared constants and deduplicate frontend utilities
Remove duplicated constants and create shared utility modules.
- [x] Extract CHIP_COLORS array and hashColor function into shared `$lib/utils/colors.ts`
- [x] Update DomainTagInput.svelte, SenderDomainTags.svelte, LabelBadges.svelte, labels/+page.svelte to import from shared module
- [x] Removed 4 duplicate definitions of CHIP_COLORS/hashColor
- [x] `npx svelte-check` returns 0 errors

### US-010: Final validation — clean build with zero warnings
Verify the entire project builds cleanly with no warnings.
- [x] `cargo check` — 0 errors, 0 warnings
- [x] `cargo clippy -- -W clippy::all` — 0 warnings
- [x] `cargo test` — 198 tests pass, 0 failures
- [x] `npx svelte-check` in apps/web — 0 errors, 0 warnings
- [x] `cargo fmt` — all formatting issues fixed

## Non-Goals
- Adding new features or functionality
- Changing database schema
- Modifying API contracts/endpoints
- Performance optimization (separate effort)
- Adding new tests (only fix existing)

## Technical Notes
- Project uses SQLite by default, PostgreSQL optional
- Frontend is SvelteKit with TypeScript
- Backend is Rust with Axum + SQLx
- 204 existing tests must continue to pass
- No breaking API changes allowed
