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
- [ ] Replace `&Box<dyn ImportConnector>` with `&dyn ImportConnector` (3 instances)
- [ ] Add `Default` impl for types with `new()` (8 instances)
- [ ] Replace `.len() == 0` with `.is_empty()` (5 instances)
- [ ] Replace `get(0)` with `first()` (2 instances)
- [ ] Fix redundant closures, manual flatten, needless range loops, and other minor lints
- [ ] `cargo clippy -p import_service -- -W clippy::all` returns 0 warnings

### US-003: Fix all cargo clippy warnings in sync_service (10 warnings)
Address the 10 clippy warnings in the sync_service crate.
- [ ] Refactor `AppState::new()` to take a config struct instead of 13 arguments
- [ ] Refactor `log_operation` to take a struct instead of 8 arguments
- [ ] Refactor `fetch_emails` to take a config/params struct instead of 8 arguments
- [ ] Replace `match ... { Ok(_) => true, Err(_) => false }` with `.is_ok()` (1 instance)
- [ ] Replace `map_or(false, ...)` with `is_some_and()` (4 instances)
- [ ] Remove unnecessary `u32` to `u32` cast (1 instance)
- [ ] Replace manual char comparison with char array (1 instance)
- [ ] `cargo clippy -p sync_service -- -W clippy::all` returns 0 warnings

### US-004: Fix clippy warnings in remaining crates (cache_layer, ai_middleware, local_store, cli_client)
Address the 16 remaining clippy warnings across 4 crates.
- [ ] cache_layer: Remove unneeded `return` statement (line 197)
- [ ] ai_middleware: Refactor `log_interaction` to take a struct instead of 9 arguments
- [ ] local_store: Replace 4 redundant closures and 1 `or_insert_with(Vec::new)` with `or_default()`
- [ ] cli_client: Replace `or_insert_with(Vec::new)` with `or_default()`
- [ ] Fix 6 compilation warnings (2 unused imports in test files, 3 unnecessary `mut`, 1 useless comparison)
- [ ] `cargo clippy -- -W clippy::all` returns 0 warnings across entire workspace

### US-005: Fix production .unwrap() calls in Rust backend
Replace `.unwrap()` in non-test code with proper error handling.
- [ ] `crates/local_store/src/repositories/contact.rs:33` — Replace `serde_json::to_string(&contact.metadata).unwrap()` with `.map_err()`
- [ ] `crates/sync_service/src/websocket.rs:137` — Replace `serde_json::to_string(&event).unwrap()` with `.unwrap_or_default()` or `.map_err()`
- [ ] `crates/sync_service/src/import_routes.rs:153` — Replace `field.file_name().unwrap()` with `.unwrap_or("unknown")`
- [ ] `cargo test` still passes (204 tests)

### US-006: Remove dead code markers in Rust backend
Remove or properly use the 28+ items marked `#[allow(dead_code)]`.
- [ ] Review and remove unused structs/fields in `sync_service/src/ws.rs` (if superseded by websocket.rs)
- [ ] Review and remove unused validation constants in `sync_service/src/validation.rs` (lines 9-23)
- [ ] Review and remove unused functions in `sync_service/src/update_system.rs`
- [ ] Review and remove unused functions in `sync_service/src/observability.rs`
- [ ] Review and remove unused functions in `sync_service/src/audit.rs`
- [ ] Review and remove unused functions in `cli_client/src/import.rs`
- [ ] `cargo check` passes with no new warnings

### US-007: Replace `any` types with proper TypeScript types in frontend
Eliminate the 45+ `any` usages across the frontend codebase.
- [ ] Type `email-domains/+page.svelte` domains array with proper interface
- [ ] Type `email-explorer/+page.svelte` domains, emails, overlaps, selectedEmail with proper interfaces
- [ ] Type `locations/+page.svelte` data objects
- [ ] Type `dashboard/+page.svelte` updateInfo and status functions
- [ ] Replace `catch (error: any)` with `catch (error: unknown)` and type guards across all files
- [ ] Type WebSocket callback parameters in api.ts
- [ ] Type optimistic store data fields
- [ ] `npx svelte-check` returns 0 errors

### US-008: Fix accessibility warnings in Svelte components (92 warnings)
Add proper ARIA attributes and roles to interactive elements.
- [ ] Add `role="button"` and `tabindex="0"` to clickable `<div>` elements (22 instances)
- [ ] Add `role="option"` to clickable `<li>` elements (6 instances)
- [ ] Add `<label>` elements or `aria-label` to form inputs (27 instances)
- [ ] Remove unused CSS selectors (concepts/+page.svelte:315, labels/[id]:501, locations:205)
- [ ] Remove unused export props from ElementSelector.svelte (subject, bodyText)
- [ ] `npx svelte-check` returns 0 warnings

### US-009: Extract shared constants and deduplicate frontend utilities
Remove duplicated constants and create shared utility modules.
- [ ] Extract CHIP_COLORS array from DomainTagInput.svelte and SenderDomainTags.svelte into shared `$lib/utils/colors.ts`
- [ ] Update both components to import from shared module
- [ ] Extract duplicate type definitions that exist in both `types.ts` and component files
- [ ] `npx svelte-check` returns 0 errors

### US-010: Final validation — clean build with zero warnings
Verify the entire project builds cleanly with no warnings.
- [ ] `cargo check` — 0 errors, 0 warnings
- [ ] `cargo clippy -- -W clippy::all` — 0 warnings
- [ ] `cargo test` — 204+ tests pass
- [ ] `npx svelte-check` in apps/web — 0 errors, 0 warnings
- [ ] `cargo fmt --check` — no formatting issues

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
