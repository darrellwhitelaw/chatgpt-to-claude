# Requirements: ChatGPT → Claude Migrator

**Defined:** 2026-02-28
**Core Value:** Drop in your ChatGPT export and end up with a well-organized Claude.ai Project folder structure that feels like your history was always there — not dumped in bulk.

## v1 Requirements

### Import

- [ ] **IMP-01**: User can drag-and-drop a ChatGPT export ZIP onto the app window to begin migration
- [ ] **IMP-02**: User can use a file picker button as an alternative to drag-and-drop
- [x] **IMP-03**: App displays a progress bar with status text during ZIP extraction and JSON streaming
- [x] **IMP-04**: App streams and parses conversations.json without loading the entire file into memory (handles GB-scale exports)
- [ ] **IMP-05**: App handles null, missing, or unexpected fields in conversations.json gracefully (schema is undocumented and changes without notice)
- [ ] **IMP-06**: App correctly reconstructs conversation message order by traversing the node-graph structure (walking parent references from current_node)

### Security

- [ ] **SEC-01**: User can enter their Anthropic API key in the app; key is stored in macOS Keychain (never written to disk or hardcoded)
- [ ] **SEC-02**: App displays the API key entry UI on first launch if no key is stored

### AI Processing

- [ ] **AI-01**: App uses the Anthropic Message Batches API to cluster conversations by topic into named project groups
- [ ] **AI-02**: App shows an estimated API cost (tokens + approximate $) to the user before the clustering batch is submitted
- [ ] **AI-03**: App generates an AI summary for each conversation (key decisions, conclusions, context)
- [ ] **AI-04**: App extracts custom instructions and system prompts from ChatGPT conversations into a separate file per project

### Preview

- [ ] **PREV-01**: App shows a preview screen with proposed project names and conversation counts before writing any output files
- [ ] **PREV-02**: User can trigger the full output generation from the preview screen with one action

### Output

- [ ] **OUT-01**: App generates a local output folder with one subfolder per AI-identified project
- [ ] **OUT-02**: Each project subfolder contains full conversation transcripts in Markdown format
- [ ] **OUT-03**: Each project subfolder contains extracted code blocks saved as separate files (with appropriate extensions)
- [ ] **OUT-04**: Each project subfolder contains images copied from the ZIP (DALL-E outputs, uploaded images)
- [ ] **OUT-05**: Each project subfolder contains an AI-generated project instructions file (theme summary + key context)
- [ ] **OUT-06**: Each project subfolder contains AI-generated per-conversation summaries alongside full transcripts
- [ ] **OUT-07**: App opens the output folder in macOS Finder when generation is complete

### UX & Performance

- [ ] **UX-01**: App provides a cancel button to abort extraction or clustering mid-operation
- [ ] **UX-02**: App UI feels native on macOS (no obvious Electron/web app aesthetic)
- [ ] **UX-03**: App remains responsive during long-running background operations (no spinning beach ball)

### Distribution

- [ ] **DIST-01**: App ships as a drag-to-install DMG
- [ ] **DIST-02**: App binary is universal (Apple Silicon + Intel x86_64)

## v2 Requirements

### Upload Automation

- **AUTO-01**: App can optionally automate Claude.ai via browser to create Projects and upload files directly (experimental; requires Playwright sidecar)
- **AUTO-02**: Browser automation is behind an opt-in "Advanced" toggle; folder export remains the default

### Distribution

- **DIST-03**: App is code-signed and notarized for Gatekeeper-clean team distribution
  - *Note: Unsigned apps can be opened by team members via right-click → Open; notarization is recommended before wider rollout*

### History & Recovery

- **HIST-01**: App shows a log of previous migration runs with output folder paths

### Enhanced Clustering

- **CLUST-01**: User can adjust the number of target project groups before clustering runs
- **CLUST-02**: User can rename or merge project clusters in the preview screen before output

## Out of Scope

| Feature | Reason |
|---------|--------|
| Direct Claude.ai Projects API upload | No public API exists; folder export is the v1 delivery mechanism |
| Windows / Linux support | Mac-first; cross-platform after open source release if community demands it |
| Migration history / undo | One-shot tool; no persistent state needed in v1 |
| Incremental / delta imports | Full migration per run; re-runs overwrite output folder |
| Real-time sync with Claude.ai | No API surface for this |
| App Store distribution | Unnecessary complexity for team distribution use case |

## Traceability

*Updated during roadmap creation — 2026-02-28.*

| Requirement | Phase | Status |
|-------------|-------|--------|
| IMP-01 | Phase 1 | Pending |
| IMP-02 | Phase 1 | Pending |
| IMP-03 | Phase 1 | Complete (01-01) |
| IMP-04 | Phase 1 | Complete (01-01) |
| IMP-05 | Phase 1 | Pending |
| IMP-06 | Phase 1 | Pending |
| SEC-01 | Phase 2 | Pending |
| SEC-02 | Phase 2 | Pending |
| AI-01 | Phase 2 | Pending |
| AI-02 | Phase 2 | Pending |
| AI-03 | Phase 2 | Pending |
| AI-04 | Phase 2 | Pending |
| PREV-01 | Phase 3 | Pending |
| PREV-02 | Phase 3 | Pending |
| OUT-01 | Phase 4 | Pending |
| OUT-02 | Phase 4 | Pending |
| OUT-03 | Phase 4 | Pending |
| OUT-04 | Phase 4 | Pending |
| OUT-05 | Phase 4 | Pending |
| OUT-06 | Phase 4 | Pending |
| OUT-07 | Phase 4 | Pending |
| UX-01 | Phase 5 | Pending |
| UX-02 | Phase 5 | Pending |
| UX-03 | Phase 5 | Pending |
| DIST-01 | Phase 5 | Pending |
| DIST-02 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-28*
*Last updated: 2026-02-28 after roadmap creation — all 26 v1 requirements mapped*
