---
phase: 02-api-key-ai-clustering
plan: "02"
subsystem: ui
tags: [zustand, typescript, tauri, ipc, state-management]

# Dependency graph
requires:
  - phase: 02-api-key-ai-clustering
    provides: Plan 02-01 keychain commands and schema columns — Rust backend foundation

provides:
  - Extended AppPhase union with 5 Phase 2 variants (awaiting-key, key-stored, cost-ready, clustering, clustering-complete)
  - Extended AppState with 4 Phase 2 fields (tokenEstimate, costEstimateUsd, batchId, clusterError)
  - 6 new Zustand actions for Phase 2 state transitions
  - ClusterEvent discriminated union type for IPC bindings
  - StartClusteringArgs type for cluster command IPC

affects:
  - 02-03 (ApiKeyScreen component needs awaiting-key and key-stored phases)
  - 02-04 (CostScreen component needs cost-ready phase and tokenEstimate/costEstimateUsd fields)
  - 02-05 (Rust cluster command emits ClusterEvent variants defined here)
  - 02-06 (ClusteringView uses clustering/clustering-complete phases and batchId)
  - App.tsx routing to Phase 2 screens

# Tech tracking
tech-stack:
  added: []
  patterns: [discriminated-union-ipc-events, zustand-phase-machine]

key-files:
  created: []
  modified:
    - src/store/appStore.ts
    - src/lib/bindings.ts

key-decisions:
  - "AppPhase is a flat union (not nested objects) — all screens can switch on a single string value; simple and exhaustive"
  - "ClusterEvent uses event discriminant (not type) to match TypeScript convention already established by IngestEvent in bindings.ts"
  - "setClusterError sets phase to 'error' (not 'clustering-error') — single error phase handles both Phase 1 and Phase 2 error UI"

patterns-established:
  - "State machine pattern: AppPhase as string union drives screen routing in App.tsx"
  - "IPC event pattern: discriminated union with event literal + optional data payload, matching Rust Channel<T> emission pattern"

requirements-completed: [SEC-02, AI-01]

# Metrics
duration: 2min
completed: 2026-02-28
---

# Phase 2 Plan 02: TypeScript Type Layer for Phase 2 Summary

**Zustand store extended with 8-variant AppPhase, 4 Phase 2 state fields, 6 new actions, and ClusterEvent IPC discriminated union in bindings.ts**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-28T20:01:46Z
- **Completed:** 2026-02-28T20:02:52Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Extended AppPhase from 4 variants to 8 variants covering the full Phase 2 lifecycle (awaiting-key → key-stored → cost-ready → clustering → clustering-complete)
- Added 4 new AppState fields (tokenEstimate, costEstimateUsd, batchId, clusterError) and 6 new Zustand actions with extended reset()
- Added ClusterEvent discriminated union (8 variants) and StartClusteringArgs to bindings.ts, in sync with planned Rust ClusterEvent structure
- TypeScript check passes with zero errors after all changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend AppPhase, AppState, and store actions in appStore.ts** - `46a334f` (feat)
2. **Task 2: Add ClusterEvent type to bindings.ts** - `cc0b62c` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/store/appStore.ts` - Extended with Phase 2 AppPhase variants, AppState fields, and Zustand actions
- `src/lib/bindings.ts` - Added ClusterEvent union and StartClusteringArgs; IngestEvent and ParseZipArgs unchanged

## Decisions Made

- AppPhase stays as a flat string union — simple to switch on, exhaustive, consistent with Phase 1 pattern
- ClusterEvent uses `event` discriminant key to match IngestEvent convention already in bindings.ts
- setClusterError routes to the existing `'error'` phase (not a new `'clustering-error'`) so App.tsx needs a single error screen for both phases

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All Phase 2 TypeScript types are defined; React screen components (Plans 02-03 ApiKeyScreen, 02-04 CostScreen, 02-06 ClusteringView) can now import from appStore.ts and bindings.ts without type errors
- Plan 02-05 Rust cluster command must emit events matching the ClusterEvent union defined here
- App.tsx routing must be extended to handle awaiting-key, key-stored, cost-ready, clustering, clustering-complete phases (Plan 02-07 or equivalent)

---
*Phase: 02-api-key-ai-clustering*
*Completed: 2026-02-28*
