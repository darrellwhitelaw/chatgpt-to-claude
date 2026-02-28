# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Drop in your ChatGPT export and end up with a well-organized Claude.ai Project folder structure that feels like your history was always there — not dumped in bulk.
**Current focus:** Phase 1 — ZIP Parsing Foundation

## Current Position

Phase: 1 of 5 (ZIP Parsing Foundation)
Plan: 2 of 4 in current phase
Status: In progress
Last activity: 2026-02-28 — Plan 01-02 complete (streaming ZIP/JSON pipeline, normalizer, SQLite insert)

Progress: [██░░░░░░░░] 10%

## Performance Metrics

**Velocity:**
- Total plans completed: 2
- Average duration: 10.5 min
- Total execution time: 21 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-zip-parsing-foundation | 2/4 | 21 min | 10.5 min |

**Recent Trend:**
- Last 5 plans: 9m, 12m
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

### Pending Todos

None.

### Blockers/Concerns

- [Research]: tauri-plugin-keyring is a community plugin — verify GitHub maintenance activity before adopting; fallback is direct Rust keyring crate
- [Research]: Tauri v2 universal binary build has known issues (tauri-apps/tauri#9748) — test Intel + Apple Silicon build early in Phase 5; have arch-specific DMG fallback ready
- [Research]: conversations.json schema changes without documentation (March 2025 attachment format change) — build parser strictly defensively from day one

## Session Continuity

Last session: 2026-02-28
Stopped at: Completed 01-02-PLAN.md — streaming ZIP/JSON pipeline, normalizer, SQLite insert, full parse_zip command
Resume file: None
