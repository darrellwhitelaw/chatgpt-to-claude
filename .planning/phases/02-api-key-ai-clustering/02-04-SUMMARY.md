---
phase: 02-api-key-ai-clustering
plan: "04"
subsystem: ui
tags: [tauri, rust, reqwest, react, typescript, zustand, keyring, anthropic]

# Dependency graph
requires:
  - phase: 02-01
    provides: keychain commands (get_api_key, set_api_key, delete_api_key), reqwest in Cargo.toml, db::get_all_conversations
  - phase: 02-02
    provides: AppPhase store with cost-ready, key-stored, clustering phases; setCostReady, setClusterError actions
  - phase: 02-03
    provides: App.tsx Phase 2 routing scaffold, ApiKeyScreen with initialError prop, key-stored placeholder
provides:
  - estimate_cost Tauri async command in cluster.rs calling /v1/messages/count_tokens
  - CostScreen React component with token + USD display in middot format
  - useCluster hook with fetchCostEstimate and INVALID_API_KEY bad-key routing
  - App.tsx cost-ready routing and useEffect auto-trigger on key-stored transition
affects:
  - 02-05 (clustering batch command — builds on cluster.rs, useCluster hook)
  - 02-06 (bad-key error recovery path through ApiKeyScreen)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "estimate_cost reads API key from Keychain inline (not app state) — security pattern"
    - "Text truncation at 8,000 chars per conversation prevents 256MB batch payload limit"
    - "INVALID_API_KEY: prefix convention for Rust-to-frontend error discrimination"
    - "Atomic Zustand setState for clusterError + phase to prevent intermediate render"
    - "useEffect on phase transition to trigger async side effects (cost estimation)"

key-files:
  created:
    - src-tauri/src/commands/cluster.rs
    - src/screens/CostScreen.tsx
    - src/hooks/useCluster.ts
  modified:
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src/App.tsx

key-decisions:
  - "useCluster bad-key path uses useAppStore.setState atomically (clusterError + phase) to avoid intermediate 'error' phase flash from setClusterError action"
  - "CostScreen Proceed uses setClustering('pending') placeholder batchId — replaced in Plan 02-05 with real batch invocation"
  - "Cancel returns to 'complete' phase via direct setState — Zustand allows direct setState outside of actions"

patterns-established:
  - "Tauri async command pattern: #[tauri::command] on async fn, State<'_> for shared state, no spawn wrapper needed"
  - "Error discrimination by string prefix: INVALID_API_KEY: prefix enables frontend to route to appropriate recovery screen"

requirements-completed: [AI-02]

# Metrics
duration: 7min
completed: 2026-02-28
---

# Phase 02 Plan 04: Cost Estimation Screen Summary

**estimate_cost Tauri command calling /v1/messages/count_tokens, CostScreen with ~2.4M tokens · estimated $1.20 format, $3.00 warning threshold, and bad-key routing back to ApiKeyScreen**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-28T20:08:03Z
- **Completed:** 2026-02-28T20:15:00Z
- **Tasks:** 2 (plus checkpoint auto-approved)
- **Files modified:** 6

## Accomplishments
- Rust `estimate_cost` command reads API key from Keychain, fetches all conversations from SQLite, truncates full_text at 8K chars, calls /v1/messages/count_tokens, computes haiku-3-5 batch cost ($0.40/$2.00 per MTok)
- 401 response clears Keychain entry and returns INVALID_API_KEY: prefixed error to frontend for routing
- CostScreen displays "{formatTokens(tokens)} tokens · estimated ${estimatedUsd.toFixed(2)}" in a single `<p>` with middot separator; amber warning callout at > $3.00
- useCluster hook wraps estimate_cost, handles INVALID_API_KEY: routing atomically to awaiting-key with clusterError
- App.tsx useEffect triggers fetchCostEstimate when phase transitions to key-stored

## Task Commits

Each task was committed atomically:

1. **Task 1: Create estimate_cost Tauri command in cluster.rs** - `0916867` (feat)
2. **Task 2: Create CostScreen component and useCluster hook; update App.tsx** - `598fea1` (feat)

**Plan metadata:** (docs: complete plan — committed after SUMMARY creation)

## Files Created/Modified
- `src-tauri/src/commands/cluster.rs` - estimate_cost async Tauri command; 401 handling; cost computation
- `src-tauri/src/commands/mod.rs` - Added `pub mod cluster;`
- `src-tauri/src/lib.rs` - Registered `commands::cluster::estimate_cost` in generate_handler!
- `src/screens/CostScreen.tsx` - Token + USD display, $3.00 warning callout, Proceed/Cancel actions
- `src/hooks/useCluster.ts` - fetchCostEstimate wrapping estimate_cost; bad-key atomic state update
- `src/App.tsx` - Import CostScreen + useCluster; useEffect on phase; cost-ready renders CostScreen

## Decisions Made
- `useCluster` bad-key path uses `useAppStore.setState({ clusterError, phase: 'awaiting-key' })` atomically rather than chaining `setClusterError` then `setAwaitingKey` — avoids intermediate 'error' phase render
- `CostScreen` `handleProceed` calls `setClustering('pending')` as placeholder batchId — Plan 02-05 will replace this with real batch invocation
- TypeScript strict mode flagged unused `setAwaitingKey` destructure (Rule 1 auto-fix: removed unused destructure, used direct setState)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused setAwaitingKey destructure in useCluster**
- **Found during:** Task 2 TypeScript verification
- **Issue:** Plan's hook template destructured `setAwaitingKey` from useAppStore but the implementation uses `useAppStore.setState` directly — TypeScript strict mode flagged it as TS6133 unused variable error
- **Fix:** Removed `setAwaitingKey` from destructure; used `useAppStore.setState({ clusterError, phase: 'awaiting-key' })` for the bad-key path (cleaner atomic update anyway)
- **Files modified:** src/hooks/useCluster.ts
- **Verification:** `npx tsc --noEmit` exits 0
- **Committed in:** `598fea1` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug/TypeScript error)
**Impact on plan:** Minor cleanup — the atomic setState approach is actually cleaner than the plan's two-step approach. No behavior change.

## Issues Encountered
None beyond the TypeScript strict mode fix documented above.

## User Setup Required
None - no external service configuration required. API key management is handled through Keychain at runtime.

## Next Phase Readiness
- estimate_cost command is ready; cluster.rs file exists for Plan 02-05 to add start_clustering command
- useCluster hook exports fetchCostEstimate — Plan 02-05 will add startClustering alongside it
- CostScreen Proceed calls setClustering('pending') — Plan 02-05 replaces with real batch invoke
- App.tsx clustering phase renders ProgressView placeholder — Plan 02-05 replaces with ClusteringView

---
*Phase: 02-api-key-ai-clustering*
*Completed: 2026-02-28*
