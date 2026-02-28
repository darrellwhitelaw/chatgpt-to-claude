# Feature Research

**Domain:** AI conversation migration tool (ChatGPT export → Claude.ai Projects)
**Researched:** 2026-02-28
**Confidence:** MEDIUM-HIGH — export schema from community reverse-engineering + OpenAI forums; Claude.ai limits from official docs; Projects API gap confirmed from official docs

---

## Critical Finding: The Integration Gap

**Claude.ai Projects do NOT have a public programmatic API.** Confirmed via official Anthropic API docs (platform.claude.com/docs) and Claude Help Center. The Anthropic API exposes Messages, Files (beta), Message Batches, and Token Counting — but zero endpoints for creating Claude.ai Projects or uploading to a Project's knowledge base.

**What this means for the app:**

The app cannot push files directly into a user's Claude.ai Project via API. The output must be a **local folder of well-named files** that the user drags into Claude.ai Projects manually — or the app must automate the browser via Playwright/Tauri WebView. Both paths are valid; the "one-click upload" in PROJECT.md requirements is either (a) browser automation, or (b) a UX handoff pattern.

This distinction shapes every feature below.

---

## OpenAI Export Format — Documented Schema

### ZIP Contents

The ChatGPT data export ZIP (requested via Settings → Data Controls → Export Data) contains:

| File/Folder | Contents | Notes |
|-------------|----------|-------|
| `conversations.json` | Full conversation history, all metadata | Primary data source; machine-readable |
| `chat.html` | Browser-viewable version with formatting | Not useful for parsing; ignore |
| Image/media files | DALL-E generated images, uploaded files | Naming conventions changed in March 2025 |

**Export sizes:** Range from a few MB (text-only) to 600MB+ (images included). Users with heavy DALL-E usage report downloads failing/stalling above 425MB.

### conversations.json Schema

Top-level structure is a **JSON array** of conversation objects:

```json
[
  {
    "id": "<uuid>",
    "title": "A320 Hydraulic System Failure.",
    "create_time": 1682368832.626937,
    "update_time": 1682369104.0,
    "current_node": "<leaf-node-uuid>",
    "mapping": {
      "<node-uuid>": {
        "id": "<node-uuid>",
        "message": {
          "id": "<message-uuid>",
          "author": {
            "role": "user",       // "user" | "assistant" | "system" | "tool"
            "name": null,          // e.g. "dalle.text2im" for DALL-E tool
            "metadata": {}
          },
          "create_time": 1682368832.626937,
          "update_time": null,
          "content": {
            "content_type": "text",  // see Content Types below
            "parts": ["message text here"]
          },
          "status": "finished_successfully",
          "end_turn": true,
          "weight": 1.0,
          "metadata": {
            "timestamp_": "absolute",
            "message_type": null,
            "model_slug": "gpt-4",
            "finish_details": { "type": "stop", "stop_tokens": [100260] }
          },
          "recipient": "all"
        },
        "parent": "<parent-node-uuid>",
        "children": ["<child-node-uuid>"]
      }
    },
    "moderation_results": [],
    "plugin_ids": null
  }
]
```

### Content Types in `message.content.content_type`

| content_type | Description | parts structure |
|---|---|---|
| `text` | Standard text message | Array of strings |
| `multimodal_text` | Text + inline images | Array of strings and image objects |
| `image_asset_pointer` | DALL-E generated image reference | Object with `asset_pointer`, `size_bytes`, `width`, `height`, `metadata.dalle` |
| `code` | Code block output (interpreter) | Object with `language`, `text` |
| `tether_browse_display` | Web browsing result | Object with URL and excerpt |
| `canvas` | Canvas document/code | Object with `content_type`: `"document"` or `"code/<lang>"` or `"webview"` |

### Tree Traversal

Conversations are **trees, not flat arrays**. To reconstruct the canonical conversation thread:
1. Start at `current_node` (the leaf/final message)
2. Walk `parent` references backwards to the root
3. Reverse the collected list → chronological order

Branching exists when users edit prompts (parent has multiple children). The `current_node` points to the chosen branch only.

### Author Roles

- `"user"` — human messages
- `"assistant"` — ChatGPT responses
- `"system"` — system prompts (usually hidden; weight = 0 suppresses display)
- `"tool"` — tool calls (DALL-E, Code Interpreter, web search); name field identifies the specific tool

### Schema Stability Warning (LOW confidence)

OpenAI has changed the export format without documentation multiple times. The most recent breaking change (March 2025) switched attachment files to `.dat` extensions and removed UUID-containing folder structure. **Parser must be tolerant of missing/null fields throughout.**

Sources: [OpenAI Community — Questions about JSON structure](https://community.openai.com/t/questions-about-the-json-structures-in-the-exported-conversations-json/954762), [Export data organization changed](https://community.openai.com/t/chatgpt-export-data-organization-has-changed-again-with-no-documentation/1161967), [Decoding exported data](https://community.openai.com/t/decoding-exported-data-by-parsing-conversations-json-and-or-chat-html/403144), [sanand0/openai-conversations](https://github.com/sanand0/openai-conversations)

---

## Claude.ai Projects — Upload Limits

### What Claude.ai Projects Accept (UI Upload)

| Category | Supported Formats |
|---|---|
| Documents | PDF, DOCX, CSV, TXT, HTML, ODT, RTF, EPUB |
| Images | JPEG, PNG, GIF, WebP |
| Audio | MP3, WAV (for transcription) |

**Per-file limit:** 30 MB
**File count:** Unlimited (paid plans)
**Total capacity:** Capped by 200,000 token context window; RAG mode auto-activates on paid plans when approaching limit, expanding effective capacity up to ~10x

**Critical constraint:** Project files use **text extraction only**, except for PDFs. Embedded images in DOCX/TXT files are not processed. PDFs under 100 pages get text + visual analysis; PDFs over 1000 pages get text only.

### Anthropic Files API (Beta — NOT Claude.ai Projects)

The Files API (`POST /v1/files`, beta header `anthropic-beta: files-api-2025-04-14`) uploads files for use in **API Messages calls only** — not Claude.ai Project knowledge bases. This is a different system.

| Limit | Value |
|---|---|
| Max file size | 500 MB per file |
| Total storage | 100 GB per organization |
| Rate limit (beta) | ~100 requests/minute |
| File persistence | Until explicitly deleted |

Files API supports: PDF, plain text, JPEG, PNG, GIF, WebP.

**Conclusion:** The app's "upload to Claude.ai Projects" feature requires either (a) browser automation against the claude.ai web UI, or (b) generating a local folder of files the user manually drags in. There is no Projects API endpoint.

Sources: [Anthropic Files API docs](https://platform.claude.com/docs/en/build-with-claude/files), [Claude Help Center — Uploading files](https://support.claude.com/en/articles/8241126-uploading-files-to-claude), [Claude Projects help](https://support.claude.com/en/articles/9519177-how-can-i-create-and-manage-projects)

---

## Existing Tools Survey

| Tool | Type | What It Does | Gaps |
|---|---|---|---|
| GPT2Claude Migration Kit | Browser bookmarklet | Exports ChatGPT memories, conversations, instructions as JSON/MD; prompts user to paste into Claude manually | No clustering, no Project creation, manual paste process |
| chatgpt-history-export-to-md (convoviz) | Python CLI | Converts ZIP to per-conversation Markdown files; inline media support | No AI clustering, no Claude upload, CLI only |
| ChatGPT_Conversations_To_Markdown | Python script | Converts to Markdown with date-folder structure (Obsidian-friendly) | No AI organization, no Claude integration |
| ai-chat-md-export | CLI tool | ChatGPT + Claude → Markdown, offline | No AI clustering, no Claude Projects upload |
| chatgpt-exporter (pionxzh) | Browser userscript | Exports individual conversations in-browser | Single conversation at a time, not bulk |
| chat-export (Trifall) | Browser extension | ChatGPT + Claude → Markdown/XML/JSON | No clustering, no Projects integration |
| ChatGPT Consolidator | Extension | Merges and organizes chat threads | Manual process, no AI clustering |

**What none of these do:** AI-based topic clustering, bulk export → organized Claude.ai Projects creation, preview-before-upload workflow, or native Mac desktop experience.

Sources: [GPT2Claude Migration Kit](https://github.com/Siamsnus/GPT2Claude-Migration-Kit), [convoviz](https://github.com/mohamed-chs/chatgpt-history-export-to-md), [ChatGPT_Conversations_To_Markdown](https://github.com/daugaard47/ChatGPT_Conversations_To_Markdown), [ai-chat-md-export](https://github.com/sugurutakahashi-1234/ai-chat-md-export), [chatgpt-exporter](https://github.com/pionxzh/chatgpt-exporter)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Accept ChatGPT export ZIP via drag-and-drop or file picker | Primary input mechanism; zero-friction entry | LOW | ZIP extraction + conversations.json parse |
| Stream-parse conversations.json without loading full file into RAM | Exports can be 600MB+; naive JSON.parse crashes or stalls | HIGH | Must use streaming JSON parser (e.g. stream-json or jsonstream); critical for large exports |
| Display conversation count, date range, and total size after import | User needs to understand what they're working with before doing anything | LOW | Computed from conversations.json metadata |
| Show proposed project groupings with conversation counts before any upload | User must preview and approve — never silently mutate | MEDIUM | AI clustering result rendered as card/list UI |
| Generate one Markdown file per conversation | Standard output format for Claude.ai upload; human-readable fallback | MEDIUM | Tree traversal + role-labeled message rendering |
| Include code blocks with language-tagged fenced syntax in Markdown output | Code is a primary reason users value ChatGPT history | LOW | Map `content_type: code` → fenced Markdown blocks |
| Preserve timestamps per message | Users want to know when conversations happened | LOW | Unix timestamp → ISO 8601 string |
| Handle conversations with no messages gracefully | Corrupt/partial exports exist; must not crash | LOW | Null checks on message field throughout parser |
| Show clear progress during processing | Large exports take time; silent progress = user abandonment | LOW | Per-phase progress bar (parsing, clustering, generating, uploading) |
| Store Claude API key in macOS Keychain, never in files | Security expectation for any tool handling API keys | LOW | Keychain APIs available in Tauri + native Swift |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| AI topic clustering into logical project groups | Core value prop: history emerges organized rather than dumped | HIGH | Send conversation titles + first messages to Claude API; cluster into N projects; let user rename/merge before proceeding |
| AI-generated project summaries as Claude Project custom instructions | Each Project gets a system prompt summarizing what it contains — so Claude in that Project has instant context | MEDIUM | Use Claude API to generate 2-3 paragraph summary per cluster; write as project instructions file |
| AI-extracted code snippets as a dedicated knowledge file per project | Code buried in chat threads becomes a searchable, reusable artifact | MEDIUM | Extract all code blocks across cluster → single `code-archive.md` per project |
| Conversation preview panel with rendered Markdown | User can read any conversation before deciding to include it | MEDIUM | Markdown renderer in the preview pane |
| Rename/merge/split proposed project groupings before upload | AI clustering is imperfect; user override is essential | MEDIUM | Drag conversations between project buckets; rename project cards |
| One-click folder export → ready-to-drag-into-Claude.ai | Produces output folder mirroring project structure; user drags folder contents into Claude.ai Project | LOW | Simple file system write; no browser automation risk |
| Per-conversation summary prepended to each transcript file | Claude processes up to 30MB per file; long conversations benefit from a TL;DR header for indexing | MEDIUM | Use Claude API to summarize each conversation in 2-3 sentences |
| Filter conversations by date range before processing | User may only want recent history, not 3-year archive | LOW | Date picker against `create_time` field |
| Detect and skip tool-only messages (DALL-E, code interpreter internals) | Raw tool messages are noise in Claude context | LOW | Filter `author.role == "tool"` or weight = 0 from rendering |
| Handle branching conversation trees correctly | Parser must follow `current_node` path, not just flatten all nodes | MEDIUM | Many tools get this wrong; correctly implemented it's a differentiator in quality |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Automated browser login to claude.ai to upload files | "One-click upload" sounds great | Fragile against UI changes; violates ToS gray area; breaks when Anthropic updates claude.ai; Playwright automation requires Chrome/Chromium bundle (adds 100MB+) | Generate output folder + guide user to drag-and-drop; takes 30 seconds and never breaks |
| Full migration history / undo log | "What if something goes wrong?" | Adds persistent state complexity; this is a one-shot tool; history implies database, state management, recovery logic | Show "what would be created" preview before any write; user can re-run at any time |
| Local searchable conversation library | "Let me search my history before migrating" | That's a different product (Obsidian, Notion, etc.); scope creep that doubles build time | Focus on migration; export-to-Markdown output is importable into any note-taking tool |
| Windows / Linux support in v1 | Wider user base | Tauri or Electron needed; native macOS Keychain APIs don't map cleanly; doubles QA surface | Mac-first, open source later for community cross-platform ports |
| Real-time sync / incremental import | "Don't make me re-run the whole thing" | OpenAI export is a point-in-time snapshot, not a streaming feed; no API to detect new conversations | Full re-run is fast enough; document that users re-export when they want fresh sync |
| Import images/DALL-E output into Claude Projects | "Don't lose my images" | Claude.ai Projects do text extraction only for non-PDF files; images in TXT/Markdown are not processed; even if files are added, images won't be indexed | Note in UI that images are referenced in transcripts but not uploaded; DALL-E prompts are preserved as text |
| Automatic model-slug-based tagging | "Label which GPT model generated each response" | `model_slug` field is present but unreliable/inconsistent across export versions; adds noise to migrated content | Skip; the conversation content is what matters, not GPT-4 vs GPT-4o labels |

---

## Feature Dependencies

```
[ZIP ingestion + stream parser]
    └──requires──> [Tree traversal / conversation reconstructor]
                       └──requires──> [Content type handler]
                                          └──requires──> [Markdown renderer]

[AI topic clustering]
    └──requires──> [Claude API integration (key storage, call, parse)]
    └──requires──> [Conversation list with titles + timestamps]

[Project preview UI]
    └──requires──> [AI topic clustering result]
    └──enhances──> [Rename/merge/split buckets]

[Output folder generation]
    └──requires──> [Markdown renderer]
    └──requires──> [Project structure (from clustering or manual)]
    └──enhances──> [Per-conversation AI summary]
    └──enhances──> [AI-generated project instructions file]

[AI-extracted code archive per project]
    └──requires──> [Content type handler recognizes code blocks]
    └──requires──> [Project structure finalized]

[Date range filter]
    └──enhances──> [ZIP ingestion] (filters before clustering)
    └──conflicts──> [Full-history clustering] (partial data means worse clusters)
```

### Dependency Notes

- **Tree traversal requires content type handler:** Rendering a conversation correctly requires knowing how to turn each `content_type` variant into readable Markdown.
- **AI clustering requires Claude API integration:** The clustering step calls Claude with conversation titles; this is the only external dependency. Must work before preview UI can be built.
- **Output folder generation is independent of upload method:** Whether the user drags files manually or a future browser automation layer handles it, the output folder is always generated first.

---

## MVP Definition

### Launch With (v1)

Minimum viable product that delivers the core value.

- [ ] ZIP drag-and-drop with stream parsing of conversations.json — without this, nothing works
- [ ] Tree traversal + content type handling → per-conversation Markdown files — the fundamental output artifact
- [ ] Claude API integration with macOS Keychain storage — enables clustering and summaries
- [ ] AI topic clustering into 5-15 named project groups — the key differentiator vs every existing tool
- [ ] Preview screen: project names, conversation counts, example titles — user must see before committing
- [ ] Output folder generation with project subfolder structure — the deliverable the user takes to Claude.ai
- [ ] Per-project AI-generated instructions file (summary as custom instructions) — makes each Claude Project immediately useful
- [ ] Progress feedback across all phases — streaming/chunking is required for large exports

### Add After Validation (v1.x)

- [ ] Rename/merge/split project groupings in preview — add when user feedback shows clustering quality needs human override
- [ ] Per-conversation AI summaries prepended to transcripts — adds API cost per migration; validate users want it first
- [ ] Date range filter — add when users report needing to exclude old conversations
- [ ] Code archive extraction per project — add when developer users specifically request it

### Future Consideration (v2+)

- [ ] Browser automation upload to Claude.ai — defer; output folder + manual drag is acceptable v1 UX; automation is brittle
- [ ] Windows/Linux support — defer to potential open source community contribution
- [ ] Multiple export format support (Claude.ai export, Gemini export) — validate ChatGPT migration first, then expand
- [ ] Incremental/delta imports — not feasible with OpenAI's point-in-time export model

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Stream-parse ZIP / conversations.json | HIGH | HIGH | P1 |
| Tree traversal → Markdown per conversation | HIGH | MEDIUM | P1 |
| Claude API key via Keychain | HIGH | LOW | P1 |
| AI topic clustering | HIGH | MEDIUM | P1 |
| Preview UI (project cards + conversation list) | HIGH | MEDIUM | P1 |
| Output folder generation | HIGH | LOW | P1 |
| AI-generated project instructions | MEDIUM | LOW | P1 |
| Progress feedback / streaming UI | HIGH | LOW | P1 |
| Rename/merge clusters in preview | MEDIUM | MEDIUM | P2 |
| Per-conversation AI summaries | MEDIUM | MEDIUM | P2 |
| Date range filter | MEDIUM | LOW | P2 |
| Code archive extraction | MEDIUM | LOW | P2 |
| Conversation preview pane | MEDIUM | MEDIUM | P2 |
| Browser automation upload | HIGH want / LOW need | HIGH + brittle | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | GPT2Claude Migration Kit | convoviz / md exporters | Our Approach |
|---------|--------------|--------------|--------------|
| Input format | Browser session (live account) | ZIP file | ZIP file (offline, no account required) |
| AI topic clustering | None | None | Claude API clusters by topic into named projects |
| Output format | chatgpt_all_conversations.json + .md | Per-conversation .md files | Per-project folder with .md transcripts + instructions file |
| Claude Projects integration | Manual paste prompts | None | Output folder ready for drag-into-Claude.ai |
| Native Mac app | No | No | Yes (Tauri or native Swift) |
| Large export handling | Stalls on large accounts (30 min) | Depends on CLI memory | Stream parser designed for 600MB+ |
| Preview before action | No | No | Yes — review clusters before generating output |
| Code extraction | No | Basic fenced blocks | Dedicated code archive per project (v1.x) |
| Image handling | Excludes | Inline media (some tools) | Reference in transcript; Claude.ai won't index anyway |

---

## Sources

- [OpenAI Help Center — Export ChatGPT history](https://help.openai.com/en/articles/7260999-how-do-i-export-my-chatgpt-history-and-data) — HIGH confidence
- [OpenAI Community — conversations.json JSON structure](https://community.openai.com/t/questions-about-the-json-structures-in-the-exported-conversations-json/954762) — MEDIUM confidence (community reverse-engineering)
- [OpenAI Community — Decoding exported data](https://community.openai.com/t/decoding-exported-data-by-parsing-conversations-json-and-or-chat-html/403144) — MEDIUM confidence
- [OpenAI Community — Export data organization changed March 2025](https://community.openai.com/t/chatgpt-export-data-organization-has-changed-again-with-no-documentation/1161967) — MEDIUM confidence
- [OpenAI Community — Export too big](https://community.openai.com/t/chatgpt-data-export-too-big-feature-in-the-options/1080111) — MEDIUM confidence (user reports)
- [sanand0/openai-conversations — Schema exploration](https://github.com/sanand0/openai-conversations) — MEDIUM confidence
- [Anthropic Files API official docs](https://platform.claude.com/docs/en/build-with-claude/files) — HIGH confidence
- [Anthropic API overview](https://platform.claude.com/docs/en/api/getting-started) — HIGH confidence (confirms no Projects endpoint)
- [Claude Help Center — Uploading files](https://support.claude.com/en/articles/8241126-uploading-files-to-claude) — HIGH confidence
- [Claude Help Center — What are projects](https://support.claude.com/en/articles/9517075-what-are-projects) — HIGH confidence
- [Claude Help Center — Create and manage projects](https://support.claude.com/en/articles/9519177-how-can-i-create-and-manage-projects) — HIGH confidence (confirms no API)
- [GPT2Claude Migration Kit](https://github.com/Siamsnus/GPT2Claude-Migration-Kit) — HIGH confidence (direct inspection)
- [convoviz / chatgpt-history-export-to-md](https://github.com/mohamed-chs/chatgpt-history-export-to-md) — HIGH confidence
- [ai-chat-md-export](https://github.com/sugurutakahashi-1234/ai-chat-md-export) — HIGH confidence

---

*Feature research for: ChatGPT → Claude.ai migration Mac desktop app*
*Researched: 2026-02-28*
