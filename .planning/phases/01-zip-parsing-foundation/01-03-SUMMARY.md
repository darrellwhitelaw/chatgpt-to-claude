---
phase: 01-zip-parsing-foundation
plan: "03"
subsystem: pipeline
tags: [rust, tdd, traversal, tree-walk, node-graph, normalizer]

# Dependency graph
requires:
  - phase: 01-02
    provides: json_parser structs (MessageNode, Message, Author, Content), normalizer stub using mapping.values(), pipeline mod.rs with traversal commented out

provides:
  - traversal::linearize_messages — walks current_node->parent chain backward, reverses to chronological order
  - traversal::should_include_message — filters user/assistant with non-empty content; excludes system/tool/memory
  - normalizer::normalize — updated to use linearize_messages instead of mapping.values() iteration
  - 5 integration tests in tests/traversal_test.rs verified against hand-crafted HashMap fixtures

affects: [01-04-fts, future-export-phase]

# Tech tracking
tech-stack:
  added: []  # No new dependencies; Rust stdlib HashMap and Vec only
  patterns:
    - "Leaf-to-root collection then Vec::reverse() — avoids prepend cost, idiomatic for parent-chain traversal"
    - "HashMap::get() (not index) for all mapping lookups — prevents panic on missing node IDs"
    - "Integration tests in src-tauri/tests/ using tauri_app_lib crate name — requires pub mod pipeline in lib.rs"
    - "TDD RED/GREEN/REFACTOR: 3 atomic commits per feature"

key-files:
  created:
    - src-tauri/src/pipeline/traversal.rs
    - src-tauri/tests/traversal_test.rs
  modified:
    - src-tauri/src/pipeline/mod.rs
    - src-tauri/src/pipeline/normalizer.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Integration tests via pub mod pipeline: plan used chatgpt_to_claude_lib crate name but actual name is tauri_app_lib; pipeline needed pub visibility in lib.rs for integration test access"
  - "Leaf-to-root collection then reverse: walking parent chain naturally collects leaf-first; Vec::reverse() at end gives O(n) chronological order vs O(n^2) prepend-based approach"
  - "mapping.values() removed from normalizer: silently wrong — includes all branches and random order; linearize_messages is the only correct approach"

patterns-established:
  - "Tree walk pattern: loop { get node by ID via HashMap::get, collect message, advance to node.parent }, reverse result"
  - "should_include_message: role check first, then content Some check, then non-empty parts check — short-circuit on each"

requirements-completed: [IMP-05, IMP-06]

# Metrics
duration: 3min
completed: 2026-02-28
---

# Phase 1 Plan 03: Tree Traversal Summary

**TDD implementation of linearize_messages: walks current_node->parent chain backward (reversing to chronological) to correctly reconstruct branched conversation order, replacing mapping.values() iteration in normalizer**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-28T17:37:14Z
- **Completed:** 2026-02-28T17:40:02Z
- **Tasks:** 2 (3 commits: test + feat + refactor)
- **Files modified:** 5

## Accomplishments

- Correct tree traversal: walks current_node -> parent -> ... -> root, collecting in leaf-first order, reverses to chronological — the only approach that handles branched conversations correctly
- Branched conversation test explicitly verifies that the "Response v1" branch is excluded when current_node points through "Response v2" — catching the mapping.values() bug
- Normalizer updated to use linearize_messages — message_count and full_text now reflect the actual conversation the user saw, not all branches combined

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — failing tests for linearize_messages** - `a476f22` (test)
2. **Task 2: GREEN — implement linearize_messages** - `4c9c5ff` (feat)
3. **Task 2: REFACTOR — wire linearize_messages into normalizer** - `c9dd812` (refactor)

**Plan metadata:** (docs commit follows this SUMMARY)

_Note: TDD tasks have 3 commits: test (RED) -> feat (GREEN) -> refactor (wiring)_

## Files Created/Modified

- `src-tauri/src/pipeline/traversal.rs` — linearize_messages (parent-chain walk, reverse to chronological) and should_include_message (role + content filter)
- `src-tauri/tests/traversal_test.rs` — 5 integration tests: linear chain, branched conversation (branch exclusion), missing node (no panic), empty mapping, system message exclusion
- `src-tauri/src/pipeline/mod.rs` — uncommented traversal module declaration (was commented as placeholder in 01-02)
- `src-tauri/src/pipeline/normalizer.rs` — replaced mapping.values() loop with linearize_messages call; message_count derived from traversal result
- `src-tauri/src/lib.rs` — made pipeline pub for integration test crate access

## Decisions Made

- **Integration test crate name:** The plan specified `chatgpt_to_claude_lib` but the actual lib name in Cargo.toml is `tauri_app_lib`. Used correct name. Also made `pipeline` pub in `lib.rs` to allow integration test access — plan implied this was needed but didn't state it explicitly.
- **Leaf-to-root then reverse:** Walking parent references naturally produces leaf-first order. Collecting into Vec then reversing once is O(n) and idiomatic. Alternative of prepending to a VecDeque would be O(n) in memory moves; reverse is simpler.
- **mapping.values() fully removed from normalizer:** The old comment in normalizer.rs said "traversal is wired after plan 01-03" — that plan is now this one, so the migration is complete.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adapted test crate name and made pipeline pub**
- **Found during:** Task 1 (RED — writing integration tests)
- **Issue:** Plan specified import `chatgpt_to_claude_lib::pipeline::traversal` but Cargo.toml names the lib `tauri_app_lib`. Pipeline module was also private (`mod pipeline`) preventing integration test access.
- **Fix:** Used `tauri_app_lib::pipeline::traversal` in test file. Changed `mod pipeline` to `pub mod pipeline` in lib.rs.
- **Files modified:** src-tauri/tests/traversal_test.rs, src-tauri/src/lib.rs
- **Verification:** cargo test compiled and ran all 5 tests
- **Committed in:** a476f22 (Task 1 RED commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary correction for tests to compile. No scope creep — same test logic, just correct crate name and visibility.

## Issues Encountered

None — cargo check and cargo test passed on first attempt after implementation. The traversal logic is straightforward: the parent-chain walk is a standard linked-list traversal pattern in Rust.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Traversal is complete and tested — linearize_messages is the authoritative source of message ordering for all downstream consumers
- Plan 01-04 (FTS) can use full_text from normalizer knowing it now reflects correct conversation order
- The IMP-05 (defensive nullability, no panic on missing nodes) and IMP-06 (correct traversal) requirements are both satisfied

---
*Phase: 01-zip-parsing-foundation*
*Completed: 2026-02-28*
