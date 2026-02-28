# Project Research Summary

**Project:** ChatGPT to Claude — Mac Desktop Migration Tool
**Domain:** AI conversation export migration (ChatGPT ZIP → Claude.ai Projects)
**Researched:** 2026-02-28
**Confidence:** MEDIUM-HIGH

## Executive Summary

This is a Mac desktop data migration tool that takes ChatGPT conversation export ZIPs and transforms them into organized, AI-clustered project files ready for import into Claude.ai. The recommended approach is a Tauri 2.x desktop app (React + TypeScript frontend, Rust backend) that streams large ZIP files without loading them into memory, uses the Claude Batch API to cluster conversations by topic, generates per-project Markdown files, and delivers a local output folder the user manually drags into Claude.ai Projects. Every existing tool in this space is either a CLI script, a browser bookmarklet, or a single-format exporter — none offer AI-based topic clustering, a preview-before-action workflow, or a native Mac experience. This app's core differentiator is turning unstructured chat history into a coherent, pre-organized Claude workspace.

The single most important research finding is that **Claude.ai Projects has no public programmatic API**. The Anthropic Files API (beta) uploads files to the API workspace only — it does not create or populate Claude.ai Projects visible in the web UI. The "upload to Claude.ai Projects" requirement in the project brief cannot be fulfilled via official API. The recommended output strategy is a well-organized local folder of Markdown files the user drags into Claude.ai Projects manually — this approach never breaks, never violates ToS, and requires no fragile browser automation. Browser automation (Playwright) is viable as a v2 experimental feature but must never be the primary output path.

The highest risks are: (1) conversations.json is a tree structure requiring explicit traversal — parsers that flatten it produce silently wrong output; (2) stream parsing is mandatory — synchronous JSON.parse on a 600MB+ export causes OOM crashes; (3) Anthropic Batch API must be used for clustering — sequential API calls hit Tier 1 rate limits at scale; and (4) browser automation against claude.ai breaks with every Anthropic UI deploy. All four are preventable by architectural decisions made in Phase 1.

---

## Key Findings

### Recommended Stack

Tauri 2.x is the clear choice over Electron or SwiftUI. Its Rust backend handles streaming ZIP and JSON parsing without memory pressure (critical for multi-GB exports), WebKit rendering gives genuine macOS-native feel unlike Electron's Chromium, and the binary is ~5-15 MB vs. Electron's ~150-200 MB. The TypeScript frontend using React 19 and shadcn/ui aligns with the team's existing skills. The `@anthropic-ai/sdk` 0.78.x handles all Claude API calls with full TypeScript types and streaming support. Zustand handles one-shot migration state. macOS Keychain integration is handled via the `tauri-plugin-keyring` community plugin or direct Rust `keyring` crate — never plaintext config files.

**Core technologies:**
- **Tauri 2.4.x**: Desktop shell — Rust backend for memory-safe file I/O, WebKit for native macOS UI
- **React 19 + TypeScript 5.x**: Frontend — team velocity, type-safe Tauri IPC bridge via `tauri-specta`
- **`unzipper` 0.12.x**: ZIP streaming — the only actively maintained, ZIP64-compliant streaming library; `adm-zip` and `node-stream-zip` are disqualified
- **`@anthropic-ai/sdk` 0.78.x**: Claude API calls — clustering, summaries, Batch API for bulk work
- **Vite 6.x + shadcn/ui + Tailwind 4.x**: Frontend tooling and components
- **Zustand 4.x**: App state — lightweight, no persistence needed for a one-shot migration tool
- **SQLite (via rusqlite/sqlx)**: Intermediate store — survives crashes, enables resume, source of truth across pipeline stages

**Critical version note:** `@anthropic-ai/sdk@^0.78` requires Node.js >=20 LTS. Tauri sidecar Node binary must match. Universal macOS builds (Intel + Apple Silicon) have known Tauri v2 issues — test early, keep arch-specific DMGs as fallback.

### Expected Features

All existing tools (GPT2Claude Migration Kit, convoviz, ai-chat-md-export, chatgpt-exporter) share the same gaps: no AI topic clustering, no project-level organization, no preview-before-action, and no native Mac experience. The opportunity is to deliver all four.

**Must have (table stakes):**
- ZIP drag-and-drop with stream parsing of conversations.json — without this, nothing works
- Tree traversal + content type handling → per-conversation Markdown files — the fundamental output artifact
- Claude API key stored in macOS Keychain — security requirement, non-negotiable from day one
- AI topic clustering into 5-15 named project groups — the core differentiator vs. every existing tool
- Preview screen: project names, conversation counts, example titles — user must approve before any file generation
- Output folder generation with project subfolder structure — what the user takes to Claude.ai
- Per-project AI-generated instructions file — makes each Claude Project immediately useful
- Progress feedback across all phases — streaming/chunking required for large exports

**Should have (competitive):**
- Rename/merge/split project groupings in preview — add when user feedback shows clustering needs human override
- Per-conversation AI summaries prepended to transcripts — validates first, costs API tokens
- Date range filter — users with 3+ years of history need a way to limit scope
- Code archive extraction per project — high value for developer users
- Conversation preview pane — user reads a conversation before deciding to include it

**Defer (v2+):**
- Browser automation upload to Claude.ai — fragile; output folder + manual drag is acceptable v1 UX
- Windows/Linux support — defer to open source community contribution
- Multiple export format support (Claude.ai export, Gemini) — validate ChatGPT migration first
- Incremental/delta imports — not feasible with OpenAI's point-in-time export model
- Full migration history/undo log — scope creep; re-run is fast enough

### Architecture Approach

The architecture is a four-stage streaming pipeline with SQLite as checkpoint store between stages: (1) ZIP streaming + JSON stream parsing in Rust → SQLite, (2) Claude Batch API clustering orchestrated from Rust → SQLite, (3) React preview UI for user approval of manifest, (4) output folder generation. The Playwright browser automation layer — if implemented — lives as an isolated Node.js sidecar process communicating over stdin/stdout JSON-RPC, never embedded in the Rust backend. This isolation means the brittle browser automation can be updated or removed without touching the rest of the architecture.

**Major components:**
1. **ZIP/JSON Pipeline (Rust)** — streaming extraction and parsing; writes each conversation to SQLite immediately; never accumulates in memory
2. **Conversation Store (SQLite)** — source of truth across all stages; enables crash recovery and resume without re-parsing
3. **Cluster Orchestrator (Rust + Batch API)** — chunks conversations, submits to Claude Batch API, polls for completion, writes cluster assignments back to SQLite
4. **Cluster Preview UI (React)** — renders ProjectManifest; allows rename/merge/exclude before any file generation
5. **Output Driver (Rust)** — generates per-project subfolder with Markdown transcripts, instructions file, and code archive
6. **Playwright Sidecar (Node.js, optional v2)** — browser automation isolated as a sidecar binary; communicates via JSON-RPC; handles claude.ai project creation if ever implemented
7. **macOS Keychain** — API key and session key storage; never plaintext; Tauri keychain plugin or direct Rust `keyring` crate

### Critical Pitfalls

1. **No Claude.ai Projects API exists** — decide the output strategy (local folder export) before writing any upload code; never use undocumented `claude.ai/api/projects` endpoints; the Files API does not substitute for Projects (see PITFALLS.md §Pitfall 1)

2. **conversations.json is a tree, not a list** — implement explicit `current_node` → parent chain traversal; write unit tests with a branched conversation fixture before building anything else on the parser; linear flattening produces silently wrong output (see PITFALLS.md §Pitfall 2)

3. **Stream parsing is mandatory, never optional** — `JSON.parse(fs.readFileSync(...))` OOM-crashes on exports >200MB; always use `serde_json::StreamDeserializer` in Rust or `stream-json` in Node.js; this is enforced by architecture, not style preference (see PITFALLS.md §Performance Traps)

4. **Tier 1 rate limits break sequential clustering at scale** — always use the Batch API; a user with 3,000 conversations at Tier 1 (50 RPM) takes 60+ minutes via sequential calls; Batch API handles 100K requests per batch at 50% cost discount (see PITFALLS.md §Pitfall 4)

5. **Browser automation against claude.ai breaks on every UI deploy** — if implemented at all, mark as experimental, use aria-label selectors (not CSS classes), add a health-check on startup, and never put it in the critical path of the main workflow (see PITFALLS.md §Pitfall 5)

---

## Implications for Roadmap

Based on combined research, the architecture has hard sequential dependencies. The build order below is derived directly from ARCHITECTURE.md §Build Order and validated by the pitfall-to-phase mapping in PITFALLS.md.

### Phase 1: ZIP Parsing Foundation
**Rationale:** Everything downstream reads from SQLite. The streaming parser and data model must be correct before any other work begins. This is also where the most dangerous pitfalls live (tree traversal, null fields, OOM).
**Delivers:** Drop a ZIP file → SQLite database of normalized conversations → conversation count and date range summary displayed in UI
**Addresses:** Table stakes features — ZIP drag-and-drop, stream parsing, conversation count display, graceful handling of null/corrupt messages
**Avoids:** OOM crash on large exports (streaming); silent data loss (tree traversal + null field defensive parsing); wrong message order (current_node traversal unit tests)
**Research flag:** STANDARD — Rust streaming JSON, SQLite via rusqlite/sqlx, and Tauri IPC are well-documented with established patterns. No phase-level research needed.

### Phase 2: Claude API Integration + Clustering
**Rationale:** Clustering is the core differentiator and the most API-complex stage. It must be built before the Preview UI (which renders its output) and before any output generation (which depends on cluster assignments). API key storage belongs here too — the Keychain integration is a prerequisite for any API call.
**Delivers:** After parsing completes, user triggers clustering → Batch API job submitted → polling with progress → ProjectManifest written to SQLite with cluster assignments
**Uses:** `@anthropic-ai/sdk` Batch API, `tauri-plugin-keyring`/Rust `keyring` crate, Zustand for phase state
**Avoids:** Tier 1 rate limit stalls (Batch API from day one); API key plaintext exposure (Keychain from day one); surprise API cost (cost estimate shown before clustering starts)
**Research flag:** STANDARD — Anthropic Batch API is well-documented with clear limits and pricing. Keychain plugin has known integration patterns for Tauri.

### Phase 3: Preview UI + Manifest Editing
**Rationale:** The preview-before-action contract is non-negotiable UX. Users must be able to see and edit proposed project groupings before any files are written. This phase builds the React UI layer on top of the manifest that Phase 2 produces.
**Delivers:** Editable project cards with conversation counts; rename projects, merge clusters, exclude conversations; confirm button triggers Phase 4
**Implements:** ClusterPreview React component, ProjectManifest persistence updates via Tauri commands
**Avoids:** Silent mutation of user data; no-undo regret; poor clustering quality driving abandonment
**Research flag:** STANDARD — React state management, Tauri invoke patterns, and optimistic UI updates are well-understood.

### Phase 4: Output Folder Generation
**Rationale:** This is the primary deliverable. The confirmed output strategy — local folder of Markdown files per project — is simpler than browser automation and never breaks. Build this cleanly before considering any automation path.
**Delivers:** Per-project subfolder with Markdown transcripts (one per conversation), AI-generated project instructions file, optional code archive; folder is ready for user to drag into Claude.ai Projects
**Uses:** Rust file I/O, Markdown renderer (Rust or via Tauri command), Claude API (for per-project instructions generation)
**Avoids:** Browser automation fragility; ToS gray area; Playwright bundle weight; output that misrepresents what it is ("Uploaded to Claude.ai" vs. "Files ready for import")
**Research flag:** STANDARD — file system operations in Tauri, Markdown generation, and Rust string formatting are standard patterns.

### Phase 5: Polish, Distribution + Code Signing
**Rationale:** macOS Gatekeeper will block an unsigned binary for every user except the developer. Code signing and notarization must be set up before any beta distribution. Error recovery, cancel button, and UX polish belong here.
**Delivers:** Signed + notarized DMG; cancel button with cleanup; retry logic for failed output writes; "looks done but isn't" checklist items resolved; temp file cleanup guaranteed
**Avoids:** Gatekeeper blocking beta testers; orphaned temp files filling SSDs; no-cancel UX on 2-hour runs
**Research flag:** STANDARD — Tauri code signing and notarization are documented in official Tauri distribution docs. Known patterns exist.

### Phase 6 (Future): Browser Automation Upload (Experimental)
**Rationale:** Only implement if user research shows manual drag-and-drop is a meaningful adoption barrier. Isolate as a sidecar so it never touches the core pipeline.
**Delivers:** Optional one-click upload to Claude.ai Projects via Playwright sidecar
**Research flag:** NEEDS RESEARCH — Session key auth patterns for claude.ai change without notice. Before implementing, research current unofficial endpoints (reference `claude-pyrojects` but verify against live claude.ai). This phase carries the highest ongoing maintenance risk of any phase.

### Phase Ordering Rationale

- Phase 1 must be first because SQLite is the only shared data store — nothing else can run without parsed data.
- Phase 2 must follow Phase 1 because clustering requires the full conversation set to be in SQLite before the Batch API job is built (partial context degrades cluster quality — see ARCHITECTURE.md §Anti-Pattern 5).
- Phase 3 must follow Phase 2 because the preview UI renders the ProjectManifest that clustering produces.
- Phase 4 must follow Phase 3 because output generation operates on the user-approved manifest.
- Phase 5 can overlap the tail of Phase 4 but must complete before any beta distribution.
- Phase 6 is deliberately deferred and isolated — it has no upstream dependencies it blocks, and its volatility should not jeopardize the stable core.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 6 (Browser Automation):** Session key auth patterns for claude.ai are undocumented and change without notice. Run a dedicated research spike to verify current DOM structure, session auth mechanism, and rate limit behavior immediately before implementation — not at planning time.

Phases with standard patterns (skip research-phase):
- **Phase 1:** Rust streaming JSON (`serde_json::StreamDeserializer`), SQLite via `rusqlite`/`sqlx`, Tauri IPC — all have official documentation and established community patterns.
- **Phase 2:** Anthropic Batch API — official docs, confirmed limits and pricing. `tauri-plugin-keyring`/Rust `keyring` crate — community plugin with documented integration.
- **Phase 3:** React + Tauri invoke — well-documented, standard patterns.
- **Phase 4:** Rust file I/O, Markdown generation — standard.
- **Phase 5:** Tauri macOS code signing and notarization — official documentation and known steps.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Tauri 2.4.x, React 19, Anthropic SDK 0.78.x all verified from official sources. `unzipper` vs. alternatives confirmed from npm. One uncertainty: `tauri-plugin-keyring` is a community plugin — verify maintenance activity before adopting. |
| Features | MEDIUM-HIGH | conversations.json schema from community reverse-engineering (OpenAI does not document it officially). Claude.ai limits from official docs. Export size behavior from user reports, not official spec. |
| Architecture | HIGH | Streaming pipeline, Batch API usage, SQLite checkpoint pattern, Playwright sidecar isolation all verified against official documentation and known patterns. No Projects API confirmed from official Anthropic docs. |
| Pitfalls | MEDIUM-HIGH | Critical limits (rate limits, file size limits) from official docs. Format pitfalls (tree traversal, null fields) from community sources that have been consistent across multiple references. Browser automation fragility is empirical/well-known. |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **conversations.json schema stability:** OpenAI changes this format without documentation. The March 2025 attachment format change is documented in community forums. Build the parser to be strictly defensive from day one and maintain a fixture suite with exports from different time periods. No resolution at planning time — ongoing mitigation only.

- **`tauri-plugin-keyring` maintenance status:** This is a community plugin. If it is not actively maintained, fall back to writing a small Tauri command directly wrapping the Rust `keyring` crate (no plugin needed). Verify GitHub activity before adopting.

- **Tauri universal binary build stability:** Known issues exist with Tauri v2 universal binaries (tracked in tauri-apps/tauri#9748). Test Intel + Apple Silicon universal build early in Phase 5. Have arch-specific DMG fallback ready.

- **Claude.ai session auth patterns for Phase 6:** The unofficial `claude-pyrojects` library documents current session key auth, but these endpoints are undocumented and can change at any Anthropic deploy. Do not design Phase 6 implementation details at planning time — run a fresh research spike immediately before building.

---

## Sources

### Primary (HIGH confidence)
- Anthropic API overview (no Projects endpoint confirmed): https://platform.claude.com/docs/en/api/overview
- Anthropic Files API (separate from Claude.ai Projects): https://platform.claude.com/docs/en/build-with-claude/files
- Anthropic Batch API (limits, pricing, latency): https://platform.claude.com/docs/en/build-with-claude/batch-processing
- Anthropic Rate Limits (Tier 1 limits verified 2026-02-28): https://platform.claude.com/docs/en/api/rate-limits
- Claude Help Center — Projects (no API confirmed): https://support.claude.com/en/articles/9519177-how-can-i-create-and-manage-projects
- Anthropic SDK TypeScript releases (v0.78.0, 2026-02-19): https://github.com/anthropics/anthropic-sdk-typescript/releases
- Tauri v2 official docs (version 2.4.2 current): https://v2.tauri.app/
- Tauri macOS code signing: https://v2.tauri.app/distribute/sign/macos/

### Secondary (MEDIUM confidence)
- OpenAI Community — conversations.json JSON structure (community reverse-engineering): https://community.openai.com/t/questions-about-the-json-structures-in-the-exported-conversations-json/954762
- OpenAI Community — Export data organization changed March 2025: https://community.openai.com/t/chatgpt-export-data-organization-has-changed-again-with-no-documentation/1161967
- OpenAI Community — Decoding exported data: https://community.openai.com/t/decoding-exported-data-by-parsing-conversations-json-and-or-chat-html/403144
- DoltHub Electron vs Tauri comparison (Nov 2025): https://www.dolthub.com/blog/2025-11-13-electron-vs-tauri/
- gethopp.app Tauri vs Electron performance: https://www.gethopp.app/blog/tauri-vs-electron
- claude-pyrojects unofficial library (session-key auth confirmed, fragility noted): https://github.com/hcevikdotpy/claude-pyrojects
- sanand0/openai-conversations (schema exploration): https://github.com/sanand0/openai-conversations

### Tertiary (LOW confidence — verify before use)
- tauri-plugin-keyring (community plugin, verify maintenance): https://github.com/HuakunShen/tauri-plugin-keyring
- Tauri universal binary discussion (known issues): https://github.com/orgs/tauri-apps/discussions/9419

---

*Research completed: 2026-02-28*
*Ready for roadmap: yes*
