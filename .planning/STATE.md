# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Drop in your ChatGPT export and end up with a well-organized Claude.ai Project folder structure that feels like your history was always there — not dumped in bulk.
**Current focus:** Phase 2 — API Key + AI Clustering

## Current Position

Phase: 2 of 5 (API Key + AI Clustering) — IN PROGRESS
Plan: 2 of 8 complete (Plan 02-02 done)
Status: Phase 2 underway — TypeScript type layer complete
Last activity: 2026-02-28 — Plan 02-02 complete; AppPhase extended to 8 variants, ClusterEvent IPC bindings added

Progress: [████████████] 24%

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: ~12 min
- Total execution time: ~56 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-zip-parsing-foundation | 4/4 | ~54 min | ~13 min |
| 02-api-key-ai-clustering | 2/8 | ~4 min | ~2 min |

**Recent Trend:**
- Last 5 plans: 9m, 12m, 3m, ~30m, 2m
- Trend: stable

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Research]: Stack chosen — Tauri 2.4.x + React 19 + TypeScript; Rust backend for streaming ZIP/JSON; SQLite as pipeline checkpoint store
- [Research]: Output strategy confirmed — local folder of Markdown files (not browser automation); no public Claude.ai Projects API exists
- [Research]: Batch API mandatory for clustering — sequential API calls hit Tier 1 rate limits at scale (3,000 conversations = 60+ min sequential)
- [Research]: conversations.json is a tree (not flat list) — current_node → parent traversal required; linear flattening produces silently wrong output
- [01-01]: Window height 528px outer (not 500px) — macOS title bar consumes ~28px; gives 500px usable content area
- [01-01]: rusqlite bundled feature — embeds SQLite so no system library dependency; required for portable DMG distribution
- [01-01]: parse_zip stub uses _path prefix — suppresses unused variable warning while keeping full IPC signature for plan 01-02
- [01-02]: Vec<u8>/Cursor for ZIP entry — ZipFile borrow-lifetime issue resolved by reading bytes then wrapping in BufReader<Cursor>
- [01-02]: into_iter() not StreamDeserializer — conversations.json is a single top-level array; StreamDeserializer is for NDJSON
- [01-02]: traversal module commented out in mod.rs — plan 01-03 adds it; prevents compile error before that plan runs
- [01-03]: Integration tests use tauri_app_lib crate name (not chatgpt_to_claude_lib) — lib name in Cargo.toml is tauri_app_lib; pipeline made pub in lib.rs for test access
- [01-03]: Leaf-to-root collection then Vec::reverse() — walks parent chain naturally collecting leaf-first, single reverse gives O(n) chronological order
- [01-03]: mapping.values() fully removed from normalizer — linearize_messages is now the authoritative message source for message_count and full_text
- [01-04]: Tauri onDragDropEvent used for drag-and-drop (not HTML5 onDrop) — HTML5 gives no filesystem paths in Tauri webview
- [01-04]: Component stubs created in Task 1 to allow tsc verification — App.tsx imports require all three component files to exist before typecheck passes
- [01-04]: Browse link uses @tauri-apps/plugin-dialog open() with zip extension filter — native macOS file picker
- [01-04]: SQLite Connection::open requires parent directory to exist — create_dir_all before DB init required on first launch (fixed post-verify)
- [01-04]: conversations.json parsed as full array (from_str::<Vec<_>>) not StreamDeserializer; year range as plain integers — fixed post-verify after real 704MB export exposed mismatch
- [Phase 02-api-key-ai-clustering]: SERVICE constant = com.darrellwhitelaw.chatgpt-to-claude matches app bundle ID for Keychain scoping
- [Phase 02-api-key-ai-clustering]: get_api_key returns Err string on NoEntry (not panic) so React can detect first-launch awaiting-key AppPhase
- [Phase 02-api-key-ai-clustering]: reqwest uses rustls-tls with default-features=false to avoid bundling OpenSSL on macOS
- [Phase 02-api-key-ai-clustering]: schema.sql uses CREATE TABLE IF NOT EXISTS so existing dev DBs need manual deletion to pick up cluster_label, summary, instructions columns
- [02-02]: AppPhase is a flat string union — simple to switch on, no nested objects, consistent with Phase 1 pattern
- [02-02]: ClusterEvent uses event discriminant key to match IngestEvent convention already in bindings.ts
- [02-02]: setClusterError routes to 'error' phase (not a new 'clustering-error') — single error phase covers both Phase 1 and Phase 2 error UI

### Pending Todos

None.

### Blockers/Concerns

- [Research]: tauri-plugin-keyring is a community plugin — verify GitHub maintenance activity before adopting; fallback is direct Rust keyring crate
- [Research]: Tauri v2 universal binary build has known issues (tauri-apps/tauri#9748) — test Intel + Apple Silicon build early in Phase 5; have arch-specific DMG fallback ready
- [Research]: conversations.json schema changes without documentation (March 2025 attachment format change) — build parser strictly defensively from day one

## Session Continuity

Last session: 2026-02-28
Stopped at: Completed 02-api-key-ai-clustering/02-02-PLAN.md — AppPhase extension + ClusterEvent IPC bindings. Ready for Plan 02-03.
Resume file: None
