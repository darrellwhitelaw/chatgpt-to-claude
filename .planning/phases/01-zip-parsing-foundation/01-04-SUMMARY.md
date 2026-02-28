---
phase: 01-zip-parsing-foundation
plan: "04"
subsystem: ui
tags: [react, tauri, zustand, typescript, drag-and-drop, lucide-react]

# Dependency graph
requires:
  - phase: 01-02
    provides: "parse_zip Tauri command with IngestEvent channel — the IPC layer this UI drives"
  - phase: 01-03
    provides: "traversal/normalizer (planned but not yet executed at time of this plan)"
provides:
  - "Zustand appStore state machine: idle | parsing | complete | error"
  - "useIngest hook: startIngest(zipPath) driving parse_zip via Tauri Channel"
  - "DropZone component: Tauri onDragDropEvent native drop + plugin-dialog file picker"
  - "ProgressView component: spinner + human-readable stage labels"
  - "SummaryCard component: 'Found N conversations (YYYY – YYYY)' + Continue button"
  - "App.tsx state machine rendering: idle→DropZone, parsing→ProgressView, complete→SummaryCard, error→DropZone+error"
affects:
  - 02-search-and-browse
  - phase-2-onward (Continue button wired in next phase)

# Tech tracking
tech-stack:
  added: []  # zustand, lucide-react, @tauri-apps/plugin-dialog already in package.json from setup
  patterns:
    - "Zustand state machine with named transitions (setStage, setError, setComplete, reset)"
    - "Tauri IPC via Channel<IngestEvent> — onmessage drives store transitions"
    - "Tauri onDragDropEvent for file drop (NOT HTML5 dataTransfer — Tauri gives no file paths via HTML5)"
    - "Component stubs created in Task 1 to allow tsc verification, replaced in Task 2"

key-files:
  created:
    - src/store/appStore.ts
    - src/hooks/useIngest.ts
    - src/components/DropZone.tsx
    - src/components/ProgressView.tsx
    - src/components/SummaryCard.tsx
  modified:
    - src/App.tsx

key-decisions:
  - "Component stubs created alongside Task 1 (Rule 3 deviation) — App.tsx imports components, they must exist for tsc to pass"
  - "Tauri onDragDropEvent used for drag-and-drop (not HTML5 onDrop) — HTML5 gives no filesystem paths in Tauri"
  - "Browse link uses @tauri-apps/plugin-dialog open() with zip extension filter"
  - "Inline error shown adjacent to drop zone; zone resets and stays interactive (no dedicated error screen)"
  - "Stage labels are human-readable strings only — no byte counts, no file paths, no technical jargon"

patterns-established:
  - "State machine pattern: AppPhase enum drives component rendering in App.tsx"
  - "Named store transitions (setStage/setError/setComplete) rather than direct state mutation"
  - "Tauri drag-and-drop: useEffect + getCurrentWindow().onDragDropEvent() + cleanup via returned unlisten fn"

requirements-completed: [IMP-01, IMP-02, IMP-03]

# Metrics
duration: 13min
completed: 2026-02-28
---

# Phase 01 Plan 04: Import UI — Drop Zone, Progress, and Summary Card Summary

**React import UI with Tauri native drag-and-drop, file picker fallback, three-stage progress spinner, and conversation count summary card driving the Rust parse_zip command**

## Performance

- **Duration:** 13 min
- **Started:** 2026-02-28T17:37:27Z
- **Completed:** 2026-02-28T17:39:57Z
- **Tasks:** 2 of 3 (stopped at checkpoint:human-verify Task 3)
- **Files modified:** 6

## Accomplishments
- Zustand state machine (idle/parsing/complete/error) with clean named transitions
- useIngest hook wiring Tauri Channel<IngestEvent> to store state across all 6 event variants
- DropZone component with Tauri native onDragDropEvent (not HTML5 drag), dashed border, Upload icon, inline error
- ProgressView showing stage labels + CSS spinner only — no numbers, no jargon
- SummaryCard showing "Found N conversations (YYYY – YYYY)" with Continue button placeholder for Phase 2

## Task Commits

Each task was committed atomically:

1. **Task 1: Zustand store, useIngest hook, and App state machine** - `9294f00` (feat)
2. **Task 2: DropZone, ProgressView, and SummaryCard components** - `1d52a4d` (feat)
3. **Task 3: Human verification checkpoint** - pending (awaiting human verify)

## Files Created/Modified
- `src/store/appStore.ts` - Zustand state machine: AppPhase, Summary, useAppStore with setStage/setError/setComplete/reset
- `src/hooks/useIngest.ts` - startIngest(zipPath): Channel<IngestEvent>, maps 6 event variants to store transitions
- `src/components/DropZone.tsx` - Tauri onDragDropEvent drop + plugin-dialog file picker, dashed border, Upload icon, inline error
- `src/components/ProgressView.tsx` - CSS spinner + human-readable stage label, no numbers
- `src/components/SummaryCard.tsx` - "Found N conversations (YYYY – YYYY)" + Continue button
- `src/App.tsx` - State machine rendering: idle→DropZone, parsing→ProgressView, complete→SummaryCard, error→DropZone+error

## Decisions Made
- Component stubs created in Task 1 (Rule 3 deviation) so App.tsx could type-check before full components were written
- Tauri's `getCurrentWindow().onDragDropEvent()` used exclusively — HTML5 drag events don't give filesystem paths in Tauri
- `@tauri-apps/plugin-dialog` `open()` with `{ extensions: ['zip'] }` filter for the Browse link
- Drop zone stays interactive on error (inline message only, no dedicated error screen) — matches locked design decision

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created component stubs in Task 1 to allow tsc verification**
- **Found during:** Task 1 (App.tsx state machine)
- **Issue:** App.tsx imports DropZone, ProgressView, SummaryCard — tsc fails if those files don't exist. Task 1's verify step requires `pnpm tsc --noEmit` to pass with zero errors.
- **Fix:** Created minimal stub implementations (single `<div />` render) for all three components so App.tsx could compile. Stubs replaced with full implementations in Task 2.
- **Files modified:** src/components/DropZone.tsx, src/components/ProgressView.tsx, src/components/SummaryCard.tsx
- **Verification:** `pnpm tsc --noEmit` passed with zero errors after stubs
- **Committed in:** `9294f00` (Task 1 commit, stubs included)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary for tsc verify gate to pass. Task 2 replaced all stubs. No scope creep.

## Issues Encountered
- None beyond the component stub deviation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Complete Phase 1 UI ready for `pnpm tauri dev` verification
- All state machine transitions wired: drop → parse → progress → summary
- Continue button logs to console — Phase 2 will wire navigation there
- Awaiting human-verify checkpoint (Task 3): run `pnpm tauri dev`, verify drag-drop, picker, progress, summary card, error handling, light theme

---
*Phase: 01-zip-parsing-foundation*
*Completed: 2026-02-28*
