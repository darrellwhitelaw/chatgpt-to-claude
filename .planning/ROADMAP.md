# Roadmap: ChatGPT → Claude Migrator

## Overview

Five sequential phases, each delivering a coherent capability that unblocks the next. The pipeline flows from raw ZIP ingestion through AI clustering, user-controlled preview, output generation, and finally a distributable macOS app. Every phase delivers something the user can verify before the next phase begins.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: ZIP Parsing Foundation** - User drops a ChatGPT export ZIP and the app streams, parses, and stores all conversations in SQLite
- [ ] **Phase 2: API Key + AI Clustering** - User enters their API key and the app clusters all conversations into named project groups via Claude Batch API
- [ ] **Phase 3: Preview + Manifest Editing** - User reviews the proposed project structure and confirms before any files are written
- [ ] **Phase 4: Output Folder Generation** - App writes the full per-project Markdown output folder the user drags into Claude.ai Projects
- [ ] **Phase 5: Polish + Distribution** - App ships as a signed, notarized DMG with cancel support, retry logic, and native macOS feel

## Phase Details

### Phase 1: ZIP Parsing Foundation
**Goal**: User can drop a ChatGPT export ZIP of any size and the app streams, parses, and stores all conversations into SQLite — producing a conversation count and date range summary without loading the file into memory
**Depends on**: Nothing (first phase)
**Requirements**: IMP-01, IMP-02, IMP-03, IMP-04, IMP-05, IMP-06
**Success Criteria** (what must be TRUE):
  1. User can drag a ZIP onto the app window and see it begin processing without any file picker interaction required
  2. User can alternatively click a button to open a file picker and select a ZIP to begin processing
  3. User can watch a progress bar with status text while the ZIP is extracted and conversations.json is streamed and parsed
  4. App correctly displays a conversation count and date range after parsing completes — even for exports containing thousands of conversations and gigabytes of attachments
  5. App handles null fields, missing keys, and malformed conversation nodes gracefully without crashing or producing silently wrong output
**Plans**: 4 plans

Plans:
- [x] 01-01-PLAN.md — Tauri scaffold: workspace, Rust backend, React 19 frontend, SQLite init, IPC bridge stub
- [ ] 01-02-PLAN.md — ZIP streaming + conversations.json stream parsing (Rust, writes to SQLite)
- [ ] 01-03-PLAN.md — Conversation node-graph traversal TDD (current_node → parent chain, branched fixture)
- [ ] 01-04-PLAN.md — Drag-and-drop + file picker UI with progress stages and summary card

### Phase 2: API Key + AI Clustering
**Goal**: User can enter their Anthropic API key (stored in macOS Keychain) and trigger AI clustering of all parsed conversations — with a cost estimate shown before submission and cluster assignments written to SQLite on completion
**Depends on**: Phase 1
**Requirements**: SEC-01, SEC-02, AI-01, AI-02, AI-03, AI-04
**Success Criteria** (what must be TRUE):
  1. User sees an API key entry UI on first launch if no key is stored; key is saved to macOS Keychain and never written to disk or hardcoded
  2. App shows an estimated API cost (token count and approximate dollar amount) before the clustering batch job is submitted
  3. After user confirms, app submits conversations to Claude Batch API and displays polling progress until the batch completes
  4. Each conversation is assigned to a named project group in SQLite when clustering finishes
  5. App generates an AI summary (key decisions, conclusions, context) and extracts custom instructions/system prompts for each conversation
**Plans**: TBD

Plans:
- [ ] 02-01: macOS Keychain integration for API key (Tauri plugin or Rust keyring crate)
- [ ] 02-02: API key entry UI — first-launch detection, key entry form, key management
- [ ] 02-03: Cost estimation — token counting, approximate dollar display before batch submission
- [ ] 02-04: Batch API clustering orchestrator — chunking, submission, polling, SQLite assignment writes
- [ ] 02-05: Per-conversation AI summary generation and system prompt extraction

### Phase 3: Preview + Manifest Editing
**Goal**: User can see the proposed project structure (project names and conversation counts) and make adjustments before any files are written to disk
**Depends on**: Phase 2
**Requirements**: PREV-01, PREV-02
**Success Criteria** (what must be TRUE):
  1. User can see all proposed project names and the number of conversations assigned to each before any output file is created
  2. User can trigger full output generation from the preview screen with a single action
**Plans**: TBD

Plans:
- [ ] 03-01: Preview screen — project cards with names, conversation counts, example conversation titles
- [ ] 03-02: Confirm action — wires preview screen to Phase 4 output pipeline

### Phase 4: Output Folder Generation
**Goal**: User gets a complete local output folder — one subfolder per project — containing Markdown transcripts, AI summaries, extracted code, images, and a project instructions file; Finder opens when generation is complete
**Depends on**: Phase 3
**Requirements**: OUT-01, OUT-02, OUT-03, OUT-04, OUT-05, OUT-06, OUT-07
**Success Criteria** (what must be TRUE):
  1. App creates a local output folder with one subfolder per AI-identified project after user confirms in the preview screen
  2. Each project subfolder contains full conversation transcripts in Markdown format and AI-generated per-conversation summaries alongside them
  3. Each project subfolder contains extracted code blocks saved as separate files with appropriate file extensions
  4. Each project subfolder contains images (DALL-E outputs, uploaded images) copied from the export ZIP
  5. Each project subfolder contains an AI-generated project instructions file (theme summary and key context)
  6. macOS Finder opens to the output folder automatically when generation is complete
**Plans**: TBD

Plans:
- [ ] 04-01: Output folder structure — per-project subfolders, Rust file I/O
- [ ] 04-02: Markdown transcript generation — per-conversation files with prepended summaries
- [ ] 04-03: Code block extraction — per-project code archive with correct file extensions
- [ ] 04-04: Image copying — DALL-E outputs and user-uploaded images from ZIP
- [ ] 04-05: Project instructions file — AI-generated theme summary and key context per project
- [ ] 04-06: Finder reveal on completion + generation progress feedback

### Phase 5: Polish + Distribution
**Goal**: App ships as a drag-to-install DMG that runs natively on Apple Silicon and Intel Macs, with a cancel button, no spinning beach ball during long operations, and a native macOS visual feel
**Depends on**: Phase 4
**Requirements**: UX-01, UX-02, UX-03, DIST-01, DIST-02
**Success Criteria** (what must be TRUE):
  1. User can cancel an in-progress extraction or clustering operation and the app cleans up temp files without crashing
  2. App remains responsive with no spinning beach ball during multi-minute ZIP parsing or API polling operations
  3. App UI looks and feels like a native macOS app — no obvious web app or Electron aesthetic
  4. App installs by dragging to Applications from a DMG and runs on both Apple Silicon and Intel x86_64 Macs
**Plans**: TBD

Plans:
- [ ] 05-01: Cancel button with mid-operation abort and temp file cleanup
- [ ] 05-02: Background operation threading — keep UI responsive during all long-running operations
- [ ] 05-03: Native macOS UI polish — typography, spacing, iconography, window chrome
- [ ] 05-04: Universal binary build (Apple Silicon + Intel) — test early, arch-specific DMG fallback ready
- [ ] 05-05: DMG packaging — drag-to-install layout, code signing, notarization

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. ZIP Parsing Foundation | 1/4 | In progress | - |
| 2. API Key + AI Clustering | 0/5 | Not started | - |
| 3. Preview + Manifest Editing | 0/2 | Not started | - |
| 4. Output Folder Generation | 0/6 | Not started | - |
| 5. Polish + Distribution | 0/5 | Not started | - |
