---
phase: 02-api-key-ai-clustering
plan: "01"
subsystem: rust-backend
tags: [keychain, sqlite, rust, tauri, reqwest]
dependency_graph:
  requires: []
  provides:
    - keychain-commands
    - cluster-schema-columns
    - update-cluster-result-fn
    - get-all-conversations-fn
  affects:
    - 02-02-PLAN.md
    - 02-03-PLAN.md
    - 02-05-PLAN.md
tech_stack:
  added:
    - keyring = "3" (apple-native feature)
    - reqwest = "0.12" (rustls-tls, no OpenSSL)
  patterns:
    - Tauri command via #[tauri::command] pub fn
    - macOS Keychain via keyring::Entry with SERVICE + USER constants
    - rusqlite params! macro for parameterized UPDATE
key_files:
  created:
    - src-tauri/src/commands/keychain.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/store/schema.sql
    - src-tauri/src/store/db.rs
decisions:
  - SERVICE constant = "com.darrellwhitelaw.chatgpt-to-claude" matches app bundle ID for Keychain scoping
  - get_api_key returns Err string on NoEntry (not panic) — React reads this to enter awaiting-key AppPhase
  - reqwest uses rustls-tls with default-features=false — avoids OpenSSL bundling on macOS
  - schema.sql uses CREATE TABLE IF NOT EXISTS so existing dev DBs need manual deletion to pick up new columns
metrics:
  duration: ~2 min
  completed: 2026-02-28
  tasks_completed: 2
  files_modified: 6
---

# Phase 02 Plan 01: Rust Backend Foundation — Keychain + Schema Summary

**One-liner:** macOS Keychain integration via keyring 3 (apple-native) with three Tauri commands, plus SQLite schema extended to 13 columns including cluster_label/summary/instructions and db.rs batch-read helpers.

## What Was Built

### Task 1: Keyring deps + keychain.rs commands (commit 6522bce)

Added `keyring = { version = "3", features = ["apple-native"] }` and `reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }` to Cargo.toml.

Created `src-tauri/src/commands/keychain.rs` with three Tauri commands:
- `get_api_key()` — reads from macOS Keychain, returns `Err(string)` on first launch (NoEntry)
- `set_api_key(key: String)` — stores API key in Keychain, never touches disk or app state
- `delete_api_key()` — removes the Keychain entry

Registered `pub mod keychain;` in `commands/mod.rs` and all three commands in `lib.rs` `generate_handler![]`.

### Task 2: Schema extension + DB helpers (commit d1916a7)

Extended `schema.sql` to 13 columns by adding `cluster_label TEXT`, `summary TEXT`, `instructions TEXT` after `project_name TEXT`.

Added to `db.rs`:
- `update_cluster_result()` — UPDATE with cluster_label, summary, instructions WHERE id
- `ConversationRow` struct — id, title, full_text, token_estimate fields all pub
- `get_all_conversations()` — SELECT ordered by created_at ASC, returns `Vec<ConversationRow>`

## Verification

`cargo check` exits 0 with no errors. Three warnings for unused functions/struct (expected at this stage — callers come in Plans 02-03 through 02-05).

Grep checks all pass:
- `keyring = { version = "3", features = ["apple-native"] }` in Cargo.toml (line 28)
- `cluster_label TEXT` in schema.sql (line 12)
- All three keychain commands in lib.rs generate_handler! (lines 35-37)

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] `src-tauri/src/commands/keychain.rs` — created
- [x] `src-tauri/Cargo.toml` — keyring + reqwest added
- [x] `src-tauri/src/commands/mod.rs` — pub mod keychain added
- [x] `src-tauri/src/lib.rs` — all three commands in generate_handler!
- [x] `src-tauri/src/store/schema.sql` — 13 columns including cluster_label, summary, instructions
- [x] `src-tauri/src/store/db.rs` — update_cluster_result and get_all_conversations present
- [x] Commit 6522bce exists
- [x] Commit d1916a7 exists

## Self-Check: PASSED
