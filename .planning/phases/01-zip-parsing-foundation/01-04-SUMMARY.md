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
    provides: "linearize_messages traversal — accurate conversation counts and message data written to SQLite"
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
  - "SQLite open requires app data dir to exist — create_dir_all before Connection::open on first launch"
  - "conversations.json is a top-level array — serde_json::from_str::<Vec<_>> not StreamDeserializer; year range as plain integers not struct"

patterns-established:
  - "State machine pattern: AppPhase enum drives component rendering in App.tsx"
  - "Named store transitions (setStage/setError/setComplete) rather than direct state mutation"
  - "Tauri drag-and-drop: useEffect + getCurrentWindow().onDragDropEvent() + cleanup via returned unlisten fn"

requirements-completed: [IMP-01, IMP-02, IMP-03]

# Metrics
duration: ~30min
completed: 2026-02-28
---

# Phase 01 Plan 04: Import UI — Drop Zone, Progress, and Summary Card Summary

**Tauri drag-and-drop import UI with Zustand state machine parsing a 704MB real ChatGPT export and displaying "Found 1,032 conversations (2023 - 2026)" — human-verified end-to-end**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-02-28T17:37:27Z
- **Completed:** 2026-02-28
- **Tasks:** 3 of 3 (including human-verify checkpoint — approved)
- **Files modified:** 6 + Rust backend bug fixes

## Accomplishments
- Complete Phase 1 UI pipeline: drop zone → progress → summary card, human-verified with a 704MB real-world export
- Zustand state machine (idle/parsing/complete/error) with clean named transitions; useIngest wires Tauri Channel<IngestEvent> across all 6 event variants
- Two post-build Rust bugs auto-fixed: SQLite first-run failure (missing app data dir) and JSON array parsing + year range serialization
- App correctly reported "Found 1,032 conversations (2023 - 2026)" — Phase 1 pipeline end-to-end confirmed working

## Task Commits

Each task was committed atomically:

1. **Task 1: Zustand store, useIngest hook, and App state machine** - `9294f00` (feat)
2. **Task 2: DropZone, ProgressView, and SummaryCard components** - `1d52a4d` (feat)
3. **Task 3: Human verification checkpoint** - approved (user verified 1,032 conversations from 704MB export)
4. **Post-verification fix: create app data dir before opening SQLite** - `d34ac71` (fix)
5. **Post-verification fix: parse JSON array correctly and fix year range serialization** - `5891a4c` (fix)

**Plan metadata (checkpoint pause):** `e712c41` (docs)

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
- SQLite `Connection::open` requires the parent directory to exist; `create_dir_all` added before DB init for first-launch safety
- `conversations.json` is a single top-level array — `serde_json::from_str::<Vec<_>>` not `StreamDeserializer`; year range fields are plain integers not a custom struct

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created component stubs in Task 1 to allow tsc verification**
- **Found during:** Task 1 (App.tsx state machine)
- **Issue:** App.tsx imports DropZone, ProgressView, SummaryCard — tsc fails if those files don't exist. Task 1's verify step requires `pnpm tsc --noEmit` to pass with zero errors.
- **Fix:** Created minimal stub implementations (single `<div />` render) for all three components so App.tsx could compile. Stubs replaced with full implementations in Task 2.
- **Files modified:** src/components/DropZone.tsx, src/components/ProgressView.tsx, src/components/SummaryCard.tsx
- **Verification:** `pnpm tsc --noEmit` passed with zero errors after stubs
- **Committed in:** `9294f00` (Task 1 commit, stubs included)

**2. [Rule 1 - Bug] SQLite database open failed on first run — missing app data directory**
- **Found during:** Post-verification testing (after Task 3 checkpoint approval)
- **Issue:** `Connection::open` fails if the parent directory of the database path does not exist; this path is created by Tauri's `app_data_dir` but not guaranteed to exist on first launch
- **Fix:** Added `fs::create_dir_all` on the parent path before calling `Connection::open`
- **Files modified:** Rust DB init module
- **Verification:** 704MB export opened successfully on first run post-fix
- **Committed in:** `d34ac71`

**3. [Rule 1 - Bug] JSON array parsed incorrectly; year range serialization mismatch**
- **Found during:** Post-verification testing — summary card showed incorrect data
- **Issue:** Parser attempted to use `StreamDeserializer` (NDJSON line-by-line format) on a single top-level JSON array; year range was serialized as a custom struct causing deserialization failure
- **Fix:** Switched to `serde_json::from_str::<Vec<ConversationNode>>` for the array; year range now uses two plain integer fields
- **Files modified:** Rust parser module
- **Verification:** App correctly reported 1,032 conversations (2023–2026) from the 704MB export
- **Committed in:** `5891a4c`

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 bugs)
**Impact on plan:** All three fixes required for correct end-to-end operation. The two post-verification bugs were caught only under real-world conditions with an actual 704MB export. No scope creep.

## Issues Encountered
- First real-world test with a 704MB archive exposed two Rust backend bugs (SQLite init and JSON parsing) not caught by unit tests. Both fixed inline as Rule 1 deviations and re-verified successfully.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 1 pipeline is fully complete and human-verified end-to-end with a real 704MB export
- SQLite populated with parsed conversations, conversation count and year range confirmed accurate
- Continue button in SummaryCard is the Phase 2 entry point — ready to wire API key + clustering UI
- Known concern: Tauri v2 universal binary build (tauri-apps/tauri#9748) — test Apple Silicon + Intel early in Phase 5

---
*Phase: 01-zip-parsing-foundation*
*Completed: 2026-02-28*
