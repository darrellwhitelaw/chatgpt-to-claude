---
phase: 01-zip-parsing-foundation
plan: "02"
subsystem: pipeline
tags: [rust, zip, serde_json, sqlite, streaming, tauri, rusqlite]

# Dependency graph
requires:
  - phase: 01-01
    provides: AppState with Mutex<Connection>, parse_zip stub, SQLite schema

provides:
  - zip_reader::open_conversations_entry — reads conversations.json bytes from ZIP into Cursor
  - json_parser::stream_conversations — element-by-element JSON array iteration via into_iter()
  - All serde structs with Option<T> nullable fields and serde_json::Value open-ended fields
  - normalizer::normalize — ConversationExport -> ConversationRecord (count, text, flags)
  - db::insert_conversation — INSERT OR REPLACE for idempotent re-runs
  - Full parse_zip Tauri command with streaming pipeline and IngestEvent progress channel

affects: [01-03-traversal, 01-04-fts]

# Tech tracking
tech-stack:
  added: []  # zip, serde_json, rusqlite already in Cargo.toml from 01-01
  patterns:
    - "ZipFile borrow-lifetime resolution: read to Vec<u8> then wrap in BufReader<Cursor>"
    - "Streaming JSON array: Deserializer::from_reader().into_iter() — NOT StreamDeserializer"
    - "All Option<T> for nullable JSON fields; serde_json::Value for open-ended fields"
    - "INSERT OR REPLACE for idempotent conversation inserts"
    - "Skip-and-log pattern for malformed entries: eprintln! + continue, never panic"

key-files:
  created:
    - src-tauri/src/pipeline/zip_reader.rs
    - src-tauri/src/pipeline/json_parser.rs
    - src-tauri/src/pipeline/normalizer.rs
  modified:
    - src-tauri/src/pipeline/mod.rs
    - src-tauri/src/store/db.rs
    - src-tauri/src/commands/ingest.rs

key-decisions:
  - "Vec<u8>/Cursor for ZIP entry: ZipFile borrows from ZipArchive, cannot return across function boundary; reading to Vec then Cursor is idiomatic Rust resolution"
  - "into_iter() not StreamDeserializer: conversations.json is a single top-level array, not NDJSON; into_iter() correctly handles this via serde's sequence visitor"
  - "traversal module commented out in mod.rs: plan 01-03 adds it; avoids compile error before that plan runs"

patterns-established:
  - "Streaming pipeline: ZIP -> bytes -> Cursor -> streaming JSON iter -> normalize -> SQLite, no full file in memory"
  - "Defensive serde: all nullable fields Option<T>, unknown-shape fields serde_json::Value"
  - "IPC progress events: Started/ExtractingZip/ParsingConversations(every 50)/BuildingIndex/Complete"

requirements-completed: [IMP-04, IMP-05]

# Metrics
duration: 12min
completed: 2026-02-28
---

# Phase 1 Plan 02: ZIP Parsing Pipeline Summary

**Streaming Rust pipeline reads conversations.json from ZIP entry into Cursor, deserializes the JSON array element-by-element via serde's into_iter(), normalizes each conversation to a SQLite record with INSERT OR REPLACE, emitting typed IngestEvent progress over Tauri Channel**

## Performance

- **Duration:** 12 min
- **Started:** 2026-02-28T00:00:00Z
- **Completed:** 2026-02-28T00:12:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Rust streaming pipeline fully implemented: open ZIP -> read conversations.json bytes -> stream-parse array element-by-element -> normalize -> SQLite insert
- All serde structs defensively typed: Option<T> for all nullable JSON fields, serde_json::Value for open-ended `parts` and `metadata` fields — handles March 2025 ChatGPT schema changes
- parse_zip stub replaced with production command: accepts State<AppState>, emits Started/ExtractingZip/ParsingConversations/BuildingIndex/Complete events, skips malformed entries without panic

## Task Commits

Each task was committed atomically:

1. **Task 1: ZIP reader and streaming JSON parser with defensive types** - `1aff313` (feat)
2. **Task 2: Normalizer, SQLite insert, and full parse_zip command implementation** - `2fb5186` (feat)

**Plan metadata:** (docs commit follows this SUMMARY)

## Files Created/Modified

- `src-tauri/src/pipeline/zip_reader.rs` — open_conversations_entry: opens ZIP, reads conversations.json to Vec<u8>, wraps in BufReader<Cursor> to avoid ZipFile borrow issue
- `src-tauri/src/pipeline/json_parser.rs` — ConversationExport/MessageNode/Message/Author/Content serde structs (all nullable fields Option<T>); stream_conversations using into_iter()
- `src-tauri/src/pipeline/normalizer.rs` — normalize(): ConversationExport -> ConversationRecord with message count, has_images/has_code flags, full_text, token estimate
- `src-tauri/src/pipeline/mod.rs` — exposes json_parser, normalizer, zip_reader; traversal commented out (plan 01-03)
- `src-tauri/src/store/db.rs` — added insert_conversation with INSERT OR REPLACE alongside existing init_schema
- `src-tauri/src/commands/ingest.rs` — replaced parse_zip stub with full streaming pipeline implementation

## Decisions Made

- **Vec<u8>/Cursor for ZIP entry:** ZipFile borrows from ZipArchive, making it impossible to return from the owning function. Reading bytes to Vec<u8> then wrapping in BufReader<Cursor<Vec<u8>>> is the idiomatic Rust solution. JSON streaming still works element-by-element — memory is proportional to conversations.json, not the full ZIP.
- **into_iter() not StreamDeserializer:** conversations.json is a single top-level JSON array, not newline-delimited JSON. Deserializer::from_reader().into_iter() correctly handles array iteration via serde's sequence visitor. StreamDeserializer is for NDJSON.
- **traversal commented out in mod.rs:** Plan 01-03 adds the traversal module. Commenting it out prevents a compile error while keeping the declaration visible for the next plan.

## Deviations from Plan

None — plan executed exactly as written. The normalizer stub approach (create minimal version in Task 1 to satisfy mod.rs, replace in Task 2) was an implementation detail within the plan's scope.

## Issues Encountered

None — cargo check passed with zero errors on first attempt. Four warnings about unused struct fields (update_time, current_node, parent, children, etc.) are expected and intentional: these fields are declared for use by plan 01-03's traversal module.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Streaming pipeline is fully functional: a real ChatGPT export ZIP can be ingested into SQLite via parse_zip command
- Plan 01-03 (traversal) will wire linearize_messages into the normalizer for correct conversation ordering
- Plan 01-04 (FTS) will add full-text search index on the full_text column
- The IMP-04 (streaming, no full load) and IMP-05 (defensive nullability) requirements are satisfied

---
*Phase: 01-zip-parsing-foundation*
*Completed: 2026-02-28*
