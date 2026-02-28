---
phase: 02-api-key-ai-clustering
plan: "05"
subsystem: ai
tags: [anthropic, batch-api, clustering, rust, tauri, reqwest, tokio, sqlite, react, zustand]

# Dependency graph
requires:
  - phase: 02-api-key-ai-clustering/02-04
    provides: estimate_cost command, CostScreen with Proceed/Cancel, useCluster hook skeleton, AppPhase clustering
  - phase: 02-api-key-ai-clustering/02-01
    provides: Keychain read/write (estimate_cost needs API key)
  - phase: 01-zip-parsing-foundation/01-02
    provides: SQLite conversations table with full_text column as AI input
provides:
  - Two-pass AI clustering pipeline: Pass 1 sync vocab discovery + Pass 2 batch submission
  - start_clustering Tauri command with Channel<ClusterEvent> streaming
  - src-tauri/src/ai/ module: batch.rs (HTTP), prompts.rs (templates)
  - ClusteringView.tsx: spinner + stage label + elapsed time display
  - Full JSONL result parsing via custom_id HashMap (not array position)
  - SQLite writes: cluster_label, summary, instructions per conversation
  - Error recovery: clustering error shows "Try again" returning to cost-ready
affects:
  - 02-06 through 02-08: clustering-complete phase is now real, not a placeholder
  - Phase 3: will read cluster_label, summary, instructions from SQLite for output generation

# Tech tracking
tech-stack:
  added:
    - "tokio = { version = '1', features = ['time'] } — for tokio::time::sleep in poll loop"
    - "Anthropic Message Batches API — POST /v1/messages/batches, poll GET, JSONL results_url"
    - "Anthropic Messages API — POST /v1/messages for synchronous Pass 1 vocab discovery"
  patterns:
    - "Two-pass clustering: Pass 1 synchronous vocab call → Pass 2 batch with vocabulary in system prompt"
    - "Custom_id-based result matching: HashMap<custom_id, results> never positional indexing"
    - "ClusterEvent Channel streaming: same pattern as IngestEvent in Phase 1"
    - "startClustering sets phase=clustering immediately before Channel setup, then direct setState for stage updates (setStage would incorrectly transition to parsing phase)"

key-files:
  created:
    - src-tauri/src/ai/mod.rs
    - src-tauri/src/ai/batch.rs
    - src-tauri/src/ai/prompts.rs
    - src/screens/ClusteringView.tsx
  modified:
    - src-tauri/src/commands/cluster.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
    - src/hooks/useCluster.ts
    - src/store/appStore.ts
    - src/screens/CostScreen.tsx
    - src/App.tsx

key-decisions:
  - "tokio time feature added explicitly to Cargo.toml — not transitively exposed despite Tauri using tokio internally"
  - "startClustering transitions to clustering phase immediately before invoking Tauri command — ensures ClusteringView is visible during Pass 1 (before batchSubmitted event)"
  - "Direct useAppStore.setState used for stage updates in clustering flow — setStage action sets phase=parsing which would break clustering phase gate in App.tsx"
  - "CostScreen now accepts onProceed prop — removes direct store coupling, enables startClustering to be passed from App.tsx"
  - "Poll loop uses tokio::time::sleep inside async Tauri command — safe because we are already in the Tauri-managed tokio reactor, Pitfall 2 panic only applies to tokio::spawn from non-async context"

patterns-established:
  - "Pattern: AI module in src-tauri/src/ai/ separates HTTP primitives (batch.rs) from prompt templates (prompts.rs)"
  - "Pattern: ClusterEvent uses #[serde(tag='event', content='data', rename_all='camelCase')] for consistent TS discriminated union"
  - "Pattern: batch result HashMap matching via custom_id before SQLite writes"

requirements-completed: [AI-01, AI-03, AI-04]

# Metrics
duration: 4min
completed: 2026-02-28
---

# Phase 2 Plan 05: AI Clustering Pipeline Summary

**Two-pass Anthropic clustering pipeline: Pass 1 sync vocab discovery + Pass 2 JSONL batch with cluster_label, summary, and instructions written to SQLite for every conversation**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-28T20:13:39Z
- **Completed:** 2026-02-28T20:17:09Z
- **Tasks:** 2 (+ 1 auto-approved checkpoint)
- **Files modified:** 10

## Accomplishments

- Created `src-tauri/src/ai/` module with `batch.rs` (Anthropic HTTP: create_batch, poll_batch, fetch_results, discover_clusters) and `prompts.rs` (PASS1_SYSTEM_PROMPT, build_pass2_system with cluster vocabulary embedded, build_pass2_user_message with 8K char truncation)
- Implemented `start_clustering` Tauri command: Pass 1 synchronous vocab call → Pass 2 batch submission → 5-second poll loop → JSONL parsing via custom_id HashMap → SQLite writes for cluster_label + summary + instructions per conversation
- Built `ClusteringView.tsx` with spinner + stage labels ("Discovering clusters...", "Clustering conversations...", "Saving results...") + elapsed time display; wired `CostScreen` Proceed to `startClustering` via `onProceed` prop; added clustering error screen with "Try again" returning to cost-ready

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ai/ module and start_clustering command** - `4fd0ed0` (feat)
2. **Task 2: ClusteringView, startClustering hook, App.tsx wiring** - `a84c55b` (feat)

## Files Created/Modified

- `src-tauri/src/ai/mod.rs` - Public module declarations for batch and prompts submodules
- `src-tauri/src/ai/batch.rs` - Anthropic API HTTP functions: create_batch, poll_batch, fetch_results, discover_clusters; BatchRequestItem/BatchParams/BatchResult/BatchResultItem types
- `src-tauri/src/ai/prompts.rs` - PASS1_SYSTEM_PROMPT, build_pass1_message, build_pass2_system (with dynamic cluster vocabulary list), build_pass2_user_message (8K char truncation)
- `src-tauri/src/commands/cluster.rs` - Added ClusterEvent enum + start_clustering command; estimate_cost unchanged
- `src-tauri/src/lib.rs` - Added `mod ai;` and registered `commands::cluster::start_clustering` in generate_handler!
- `src-tauri/Cargo.toml` - Added `tokio = { version = "1", features = ["time"] }`
- `src/screens/ClusteringView.tsx` - Spinner + stage label + elapsed time (formatElapsed: Xs / Xm Xs)
- `src/hooks/useCluster.ts` - Full hook with fetchCostEstimate + startClustering; handles all 7 ClusterEvent variants
- `src/store/appStore.ts` - Added elapsedSecs: number field (default 0, cleared in reset)
- `src/screens/CostScreen.tsx` - Accept onProceed prop, remove direct setClustering call
- `src/App.tsx` - Import ClusteringView, destructure elapsedSecs, pass onProceed=startClustering, split error screen by clusterError presence

## Decisions Made

- **tokio time feature explicit**: Tauri uses tokio internally but does not re-export `tokio::time` in a way that's directly accessible. Added `tokio = { version = "1", features = ["time"] }` explicitly to Cargo.toml.
- **Phase transition before Channel setup**: `startClustering` sets `phase: 'clustering'` immediately before creating the Channel, so ClusteringView renders while Pass 1 is running — not just after batchSubmitted.
- **Direct setState for stage updates in clustering**: `setStage` action in Zustand sets `phase: 'parsing'`, which would break ClusteringView rendering. Used `useAppStore.setState({ stage: '...' })` to update only the stage label while preserving clustering phase.
- **onProceed prop on CostScreen**: Removes direct store dependency from CostScreen, enables App.tsx to inject `startClustering` without CostScreen knowing about the hook.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] startClustering phase transition fixed to preserve clustering phase**
- **Found during:** Task 2 (useCluster hook implementation)
- **Issue:** Plan's hook called `setStage('Discovering clusters...')` on pass1Started event. The `setStage` action in appStore sets `phase: 'parsing'`, which would hide ClusteringView (only shown for `phase === 'clustering'`) and show ProgressView instead during the entire Pass 1 phase.
- **Fix:** Changed hook to set `phase: 'clustering'` immediately in `startClustering()` before invoking the Tauri command, then use `useAppStore.setState({ stage: '...' })` for all subsequent stage label updates — preserving the clustering phase throughout.
- **Files modified:** src/hooks/useCluster.ts
- **Verification:** tsc --noEmit passes; logic trace confirms ClusteringView renders from Proceed click through completion
- **Committed in:** a84c55b (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug: incorrect phase transition during clustering stage updates)
**Impact on plan:** Essential fix for correct UI behavior — without it, ClusteringView would never render during Pass 1. No scope creep.

## Issues Encountered

- `tokio` not transitively accessible from Tauri's internal dependency — required explicit Cargo.toml addition despite RESEARCH.md suggesting it might be available. Confirmed by cargo check error E0433.

## Next Phase Readiness

- clustering-complete phase now real and reachable after SQLite writes complete
- SQLite has cluster_label, summary, instructions columns ready for Phase 3 output generation
- No blockers — full pipeline from Proceed click to clustering-complete is implemented

---
*Phase: 02-api-key-ai-clustering*
*Completed: 2026-02-28*
