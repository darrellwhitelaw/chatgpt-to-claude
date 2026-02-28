# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Drop in your ChatGPT export and end up with a well-organized Claude.ai Project folder structure that feels like your history was always there — not dumped in bulk.
**Current focus:** Phase 1 — ZIP Parsing Foundation

## Current Position

Phase: 1 of 5 (ZIP Parsing Foundation)
Plan: 0 of 4 in current phase
Status: Ready to plan
Last activity: 2026-02-28 — Roadmap created; research completed; requirements defined (26 v1)

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: —
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Research]: Stack chosen — Tauri 2.4.x + React 19 + TypeScript; Rust backend for streaming ZIP/JSON; SQLite as pipeline checkpoint store
- [Research]: Output strategy confirmed — local folder of Markdown files (not browser automation); no public Claude.ai Projects API exists
- [Research]: Batch API mandatory for clustering — sequential API calls hit Tier 1 rate limits at scale (3,000 conversations = 60+ min sequential)
- [Research]: conversations.json is a tree (not flat list) — current_node → parent traversal required; linear flattening produces silently wrong output

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: tauri-plugin-keyring is a community plugin — verify GitHub maintenance activity before adopting; fallback is direct Rust keyring crate
- [Research]: Tauri v2 universal binary build has known issues (tauri-apps/tauri#9748) — test Intel + Apple Silicon build early in Phase 5; have arch-specific DMG fallback ready
- [Research]: conversations.json schema changes without documentation (March 2025 attachment format change) — build parser strictly defensively from day one

## Session Continuity

Last session: 2026-02-28
Stopped at: Roadmap created and written to disk; ready to begin planning Phase 1
Resume file: None
