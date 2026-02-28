# Architecture Research

**Domain:** Mac desktop app — ChatGPT conversation export migration to Claude.ai Projects
**Researched:** 2026-02-28
**Confidence:** HIGH (all critical decisions verified against official documentation)

---

## Critical Decision: Claude.ai Projects API

**Verdict: No official Projects management API exists. Browser automation is required.**

Confirmed by exhaustive documentation review (February 2026):

- The **Admin API** (`/v1/organizations/...`) manages org members, workspaces, and API keys only — no project creation, no knowledge base file uploads.
- The **Files API** (`/v1/files`, beta since April 2025) uploads files for use in Messages API calls. These are workspace-scoped API files — they are NOT attached to Claude.ai Projects knowledge bases. They live in a separate namespace from the UI Projects feature.
- **No `/v1/projects` endpoint exists** in any official Anthropic documentation.
- The only known working approach for programmatic Claude.ai Project creation and file upload is session-key-based browser session automation (as evidenced by the unofficial `claude-pyrojects` library on GitHub, which authenticates via `sessionKey` and wraps reverse-engineered endpoints).

**Architectural consequence:** The upload stage requires browser automation (Playwright) controlling an authenticated claude.ai browser session. This is the high-risk component of the entire architecture — it can break if Anthropic changes their internal API. Design the upload stage as an isolated, swappable module so it can be replaced if an official API is released.

---

## Standard Architecture

### System Overview

```
┌───────────────────────────────────────────────────────────────────┐
│                        Tauri Shell (macOS)                         │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                     React UI Layer                          │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │  │
│  │  │  Drop Zone  │  │ Cluster      │  │ Upload Progress  │  │  │
│  │  │  + Progress │  │ Preview UI   │  │ + Error States   │  │  │
│  │  └──────┬──────┘  └──────┬───────┘  └────────┬─────────┘  │  │
│  └─────────┼────────────────┼──────────────────────┼──────────┘  │
│            │  Tauri IPC (invoke/Channel)           │             │
│  ┌─────────▼────────────────▼──────────────────────▼──────────┐  │
│  │                    Rust Backend                             │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │  │
│  │  │ ZIP/JSON    │  │  Cluster     │  │  Upload          │  │  │
│  │  │ Pipeline    │  │  Orchestrator│  │  Driver          │  │  │
│  │  └──────┬──────┘  └──────┬───────┘  └────────┬─────────┘  │  │
│  │         │                │                   │             │  │
│  │  ┌──────▼──────┐  ┌──────▼───────┐  ┌────────▼─────────┐  │  │
│  │  │ Conversation│  │  Claude API  │  │  Playwright      │  │  │
│  │  │ Store       │  │  (Batch API) │  │  (Browser Auto)  │  │  │
│  │  │ (SQLite)    │  └──────────────┘  └──────────────────┘  │  │
│  │  └─────────────┘                                           │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────────┐  ┌───────────────────────────────────────────┐  │
│  │ macOS       │  │  External Services                        │  │
│  │ Keychain    │  │  api.anthropic.com  |  claude.ai (browser)│  │
│  └─────────────┘  └───────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| Drop Zone UI | Accept ZIP drop or file picker, show parsing progress | React + Tauri drag-drop plugin |
| ZIP/JSON Pipeline | Streaming extraction of ZIP, streaming parse of conversations.json, write to SQLite | Rust: zip crate + serde_json streaming |
| Conversation Store | Persist parsed conversations for preview and retry; source of truth during migration | SQLite via rusqlite or sqlx |
| Cluster Orchestrator | Chunk conversations, call Claude Batch API, receive cluster assignments, build project manifest | Rust + Anthropic SDK or raw HTTP |
| Cluster Preview UI | Display proposed project names, conversation counts, allow user edits before upload | React with optimistic state |
| Upload Driver | For each project in manifest: create Claude.ai Project via browser automation, upload documents | Playwright (Node.js sidecar process) |
| macOS Keychain | Store Claude API key securely at rest | Tauri keychain plugin |

---

## Recommended Project Structure

```
src-tauri/
├── src/
│   ├── main.rs                  # Tauri app entry point
│   ├── commands/
│   │   ├── ingest.rs            # ZIP extraction + parse commands
│   │   ├── cluster.rs           # Clustering orchestration commands
│   │   └── upload.rs            # Upload trigger commands
│   ├── pipeline/
│   │   ├── zip_extractor.rs     # Streaming ZIP reader
│   │   ├── json_parser.rs       # Streaming conversations.json parser
│   │   └── normalizer.rs        # ChatGPT schema → internal ConversationRecord
│   ├── store/
│   │   ├── db.rs                # SQLite connection pool
│   │   └── migrations/          # SQL migrations
│   ├── clustering/
│   │   ├── chunker.rs           # Splits conversations into Batch API batches
│   │   ├── batch_client.rs      # Anthropic Batch API calls
│   │   └── manifest.rs          # Builds ProjectManifest from cluster results
│   └── keychain.rs              # macOS Keychain read/write
src/                             # React frontend
├── components/
│   ├── DropZone.tsx
│   ├── ParseProgress.tsx
│   ├── ClusterPreview.tsx       # Editable project manifest UI
│   └── UploadProgress.tsx
├── hooks/
│   ├── useIngest.ts             # Drives ingest pipeline via Tauri invoke
│   ├── useCluster.ts            # Drives clustering, polls batch status
│   └── useUpload.ts             # Drives upload, tracks per-project progress
└── App.tsx
playwright-sidecar/              # Separate Node.js process (sidecar)
├── uploader.ts                  # Playwright automation for claude.ai
├── session.ts                   # Session key auth management
└── index.ts                     # IPC listener (stdin/stdout or HTTP)
```

### Structure Rationale

- **src-tauri/pipeline/:** ZIP and JSON parsing happen in Rust for memory safety and performance. Rust streaming iterators avoid OOM on multi-GB exports.
- **src-tauri/store/:** SQLite is the right intermediate store. It survives crashes during long migrations, enables retry without re-parsing, and lets the preview UI query the manifest without holding everything in RAM.
- **playwright-sidecar/:** Kept as a separate Node.js process because Playwright cannot run inside Rust. Tauri sidecar support allows a bundled Node binary. Isolation means the brittle browser-automation code can be updated or replaced independently.
- **src/ (React):** Frontend stays thin — it only calls Tauri commands and renders state. No business logic in the UI.

---

## Data Model

### ChatGPT Export Schema (confirmed)

```typescript
// conversations.json top-level: array of ConversationExport
interface ConversationExport {
  id: string;           // UUID
  title: string;        // conversation title
  create_time: number;  // unix timestamp (float)
  update_time: number;
  mapping: Record<string, MessageNode>;  // keyed by message id
  current_node: string; // id of the last message
}

interface MessageNode {
  id: string;
  parent: string | null;
  children: string[];
  message: Message | null;
}

interface Message {
  id: string;
  author: { role: "system" | "user" | "assistant" | "tool"; name?: string; metadata: object };
  create_time: number | null;
  content: {
    content_type: "text" | "multimodal_text" | "tether_browsing_display" | "code";
    parts: Array<string | ImageAssetPointer | object>;
  };
  metadata: {
    model_slug?: string;
    message_type?: string;
    // ...
  };
}

interface ImageAssetPointer {
  content_type: "image_asset_pointer";
  asset_pointer: string;  // file service reference
  size_bytes: number;
  width: number;
  height: number;
}
```

### Internal ConversationRecord (normalized form stored in SQLite)

```typescript
interface ConversationRecord {
  id: string;              // from ChatGPT export
  title: string;
  created_at: number;      // unix timestamp
  message_count: number;
  has_images: boolean;
  has_code: boolean;
  token_estimate: number;  // rough estimate for batch sizing
  full_text: string;       // linearized transcript (user+assistant turns)
  cluster_id: string | null;   // assigned after clustering
  project_name: string | null; // assigned after clustering
}
```

### ProjectManifest (built after clustering, drives preview + upload)

```typescript
interface ProjectManifest {
  projects: ProjectEntry[];
  generated_at: number;
  total_conversations: number;
}

interface ProjectEntry {
  id: string;                    // local UUID
  name: string;                  // AI-suggested project name (user-editable)
  description: string;           // AI-generated summary
  conversation_ids: string[];
  custom_instructions: string;   // AI-generated system prompt for this project
  estimated_tokens: number;
  upload_status: "pending" | "in_progress" | "done" | "failed";
}
```

---

## Architectural Patterns

### Pattern 1: Streaming Pipeline with Backpressure

**What:** Never load conversations.json fully into memory. Use Rust's async streaming to parse JSON array items one at a time and write each to SQLite before advancing.

**When to use:** Always — this is not optional. ChatGPT exports with years of history plus embedded image data can easily exceed available RAM.

**Trade-offs:** Slightly more complex than `JSON.parse(fs.readFileSync(...))` but prevents OOM crashes on any real-world export.

**Implementation approach:**
```rust
// Rust pseudocode — use serde_json Deserializer with StreamDeserializer
let file = File::open(&conversations_path).await?;
let reader = BufReader::new(file);
let stream = serde_json::Deserializer::from_reader(reader)
    .into_iter::<ConversationExport>();

for result in stream {
    let conv = result?;
    store.insert_conversation(normalize(conv)).await?;
    emit_progress(&window, processed_count);
}
```

### Pattern 2: Batch Clustering via Two-Phase AI Calls

**What:** Use Claude's Message Batches API to categorize conversations. Cheaper (50% off), supports up to 100,000 requests per batch. Most batches complete within 1 hour.

**Phase 1 — Title clustering:** Send all conversation titles (not full text) in a single batch. Ask Claude to assign each a cluster label from a candidate set it generates. This is fast and cheap because titles are short.

**Phase 2 — Project naming (optional refinement):** Once clusters are formed, send a representative sample of titles per cluster to generate a final project name, description, and custom instructions.

**Chunking strategy:** Each batch request = one conversation title. A 10,000 conversation export = 10,000 batch items (well within the 100,000 item limit). Use `custom_id` = conversation UUID to match results back. Batch pricing: Claude Haiku 4.5 at $0.50/MTok input, $2.50/MTok output. For titles-only batching, cost will be negligible.

**When to include full text:** Only if title-only clustering produces poor results (detected when cluster names are too generic). Send first 500 tokens of conversation as context, not the full transcript.

**Trade-offs:** 1-hour latency is acceptable for a migration tool. Real-time streaming API would be faster but costs 2x and complicates the flow significantly.

**Example batch request per conversation:**
```typescript
{
  custom_id: conv.id,
  params: {
    model: "claude-haiku-4-5",
    max_tokens: 50,
    messages: [{
      role: "user",
      content: `Assign this conversation to one of these topics: [${candidateTopics.join(", ")}]
Or suggest a new topic if none fit.

Conversation title: "${conv.title}"
First message: "${conv.firstMessage.slice(0, 200)}"

Reply with just the topic name, nothing else.`
    }]
  }
}
```

### Pattern 3: Playwright Sidecar with IPC

**What:** Run Playwright as a bundled Node.js sidecar process. Tauri spawns it on demand. Communicate via stdin/stdout JSON-RPC or a loopback HTTP server.

**When to use:** This is the upload stage — the only viable mechanism for creating Claude.ai Projects without an official API.

**Trade-offs:**
- Fragile: claude.ai UI changes can break automation. Mitigation: pin tested Playwright version, add selector-fallback logic, surface clear error messages.
- Session key management: user must supply their claude.ai session key (obtainable from browser devtools). Store in macOS Keychain via Tauri.
- No multi-account: sidecar authenticates to one claude.ai account at a time.

**Sidecar protocol:**
```typescript
// Rust sends JSON-RPC to sidecar process stdin
{ "method": "createProject", "id": "req-1", "params": { "name": "...", "description": "..." } }
// Sidecar responds on stdout
{ "id": "req-1", "result": { "projectId": "abc123", "success": true } }
{ "method": "addDocument", "id": "req-2", "params": { "projectId": "abc123", "content": "...", "filename": "..." } }
```

### Pattern 4: SQLite as Migration Checkpoint Store

**What:** Use SQLite to persist state between pipeline stages. If the app crashes mid-migration, the user can resume from the last checkpoint rather than starting over.

**When to use:** Always — a migration over thousands of conversations can take hours. Losing progress is unacceptable.

**Tables:**
- `conversations` — parsed records, cluster assignments
- `project_manifest` — generated project entries
- `upload_log` — per-project upload status and errors

---

## Data Flow

### Full Pipeline Flow

```
[User drops ZIP file]
        |
        v
[Rust: ZIP Extractor]
  - Open ZIP with streaming reader (unzip_stream / Rust zip crate)
  - Locate conversations.json by name
  - Stream entry bytes to JSON parser
        |
        v
[Rust: JSON Stream Parser]
  - serde_json StreamDeserializer over byte stream
  - Parse one ConversationExport at a time
  - Normalize to ConversationRecord
  - Emit progress events to UI via Tauri Channel
        |
        v
[SQLite: conversations table]
  - Write each record immediately
  - Never accumulate in memory
        |
        v
[UI: Parse Complete — show summary + "Start Clustering" button]
        |
        v
[Rust: Cluster Orchestrator]
  - Query all conversations from SQLite
  - Build batch of title+first-message pairs
  - POST to api.anthropic.com/v1/messages/batches
  - Poll every 60s until processing_status == "ended" (typically < 1 hour)
  - Stream results, write cluster_id + project_name back to SQLite
        |
        v
[UI: Cluster Preview]
  - Render ProjectManifest (project names, conversation counts)
  - Allow user to rename projects, merge clusters, exclude conversations
  - Confirm = update SQLite project_manifest
        |
        v
[User clicks "Upload to Claude.ai"]
        |
        v
[Rust: Upload Driver]
  - Spawn Playwright sidecar (Node.js) if not running
  - For each project in manifest (status = "pending"):
    - Send createProject to sidecar
    - For each conversation in project:
      - Format conversation as markdown document
      - Send addDocument to sidecar
    - Send setCustomInstructions to sidecar
    - Update upload_log status to "done"
  - Emit per-project progress to UI
        |
        v
[UI: Upload Complete — show results, any errors]
```

### State Management

```
[Tauri State]
  migration_state: MigrationState (managed Rust state)
    - phase: Idle | Parsing | Clustering | Preview | Uploading | Done
    - parsed_count: u32
    - cluster_batch_id: Option<String>
    - manifest: Option<ProjectManifest>

[React UI]
  useIngest: subscribes to Tauri Channel "parse_progress"
  useCluster: polls cluster_status command every 30s
  useUpload: subscribes to Tauri Channel "upload_progress"
```

---

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| api.anthropic.com (Batch API) | REST HTTP from Rust backend | `POST /v1/messages/batches` with `anthropic-beta` header; poll `GET /v1/messages/batches/{id}`; 50% cost discount; up to 100K requests/batch; most complete < 1hr |
| api.anthropic.com (Files API) | REST HTTP, beta header `files-api-2025-04-14` | Optional: upload conversation transcripts as persistent files. Does NOT link to Claude.ai Projects knowledge base. Only useful if building a separate API-based workflow. |
| claude.ai (browser) | Playwright automation via sidecar | Unofficial session-key auth. Fragile. Only path to Projects creation. Session key expires; handle re-auth gracefully. |
| macOS Keychain | Tauri `tauri-plugin-keychain` | Store Claude API key and claude.ai session key. Never store in plaintext files or app state. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| React UI ↔ Rust backend | Tauri `invoke()` + `Channel` | Commands for actions; channels for streaming progress events. Use typed Tauri commands — no stringly typed params. |
| Rust backend ↔ Playwright sidecar | JSON-RPC over stdin/stdout | Sidecar is a Tauri sidecar binary. Tauri handles process lifecycle. Keep protocol minimal. |
| Pipeline stages ↔ SQLite | Direct rusqlite/sqlx calls within Rust | No shared state in memory — SQLite is the only source of truth across stages. |
| Cluster Orchestrator ↔ Batch API | HTTP polling loop | Do not busy-poll — 60-second intervals. Store batch_id in SQLite so crashes during clustering are resumable. |

---

## Anti-Patterns

### Anti-Pattern 1: Loading conversations.json Into Memory

**What people do:** `let data = JSON.parse(fs.readFileSync('conversations.json', 'utf8'))` or equivalent.

**Why it's wrong:** A multi-year export with images can be 500MB+ of JSON. Node.js and Rust both have limits on single-allocation sizes. The app will OOM or become unresponsive on real-world inputs. The PROJECT.md explicitly calls out this risk.

**Do this instead:** Rust streaming with `serde_json::StreamDeserializer`. Write each conversation to SQLite immediately. Never accumulate more than one conversation in memory at a time.

---

### Anti-Pattern 2: Using the Anthropic Files API as a Claude.ai Projects Substitute

**What people do:** Upload transcripts to `POST /v1/files` and assume they'll appear in Claude.ai Projects.

**Why it's wrong:** The Files API is scoped to the Anthropic API workspace. These files are used in programmatic Messages API calls only. They have no relationship to Claude.ai's UI Projects feature, which has its own private knowledge base storage. These are two separate systems.

**Do this instead:** Use Playwright browser automation to upload documents to Claude.ai Projects through the UI. Keep Files API usage separate if you want to use it for the clustering/summarization step.

---

### Anti-Pattern 3: Embedding Playwright Logic in the Rust Backend

**What people do:** Try to call Playwright from Rust via FFI or shell-exec.

**Why it's wrong:** Playwright is a Node.js library. It requires a JavaScript runtime and a browser binary. Trying to drive it from Rust directly adds enormous complexity and fragility.

**Do this instead:** Use Tauri's sidecar feature to bundle a compiled Node.js binary with Playwright pre-installed. The Rust backend communicates with the sidecar over stdin/stdout. This keeps each layer in its native runtime.

---

### Anti-Pattern 4: Treating Upload as Atomic

**What people do:** Loop over all projects and upload, assuming no failures.

**Why it's wrong:** Claude.ai session keys expire, rate limits exist on the UI, and network failures happen. A 200-project migration with no checkpointing means starting over on failure.

**Do this instead:** Track `upload_status` per project in SQLite. The upload driver processes one project at a time, marks it `done` before moving to the next. Crashes or session expiry can be resumed from the last incomplete project.

---

### Anti-Pattern 5: Firing Batch Clustering Before Parsing Completes

**What people do:** Start sending conversations to the Batch API as they are parsed to "pipeline" the work.

**Why it's wrong:** The clustering prompt benefits from knowing the full set of conversations so it can generate coherent, non-overlapping cluster names. Partial context produces fragmented clusters.

**Do this instead:** Parse completely to SQLite first. Then query the full set for clustering. Parsing is fast (minutes even for large exports). Clustering has a hard 1-hour minimum wait anyway.

---

## Scaling Considerations

This is a single-user desktop app. "Scaling" means handling larger exports gracefully, not user scale.

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 100-1,000 conversations | Everything works as designed. Clustering batch completes in minutes. |
| 1,000-10,000 conversations | Parsing and streaming still fine. Batch API handles 10K items easily. Upload stage is the bottleneck — Playwright creates projects one at a time. Expect 30-120 minutes total. |
| 10,000+ conversations | Batch API limit of 100K/batch is safe. SQLite handles millions of rows. Upload becomes very slow (hours). Consider parallelizing Playwright or adding a "select subset" filter in the Preview UI. |
| Gigabyte-sized exports | ZIP streaming + JSON streaming handles this without modification. SQLite writes are fast. Only risk is if individual conversation transcripts themselves are massive (multi-hour coding sessions). Token-truncate transcripts at normalization time if > 100K chars. |

### Scaling Priorities

1. **First bottleneck:** Playwright upload speed. Each project requires multiple sequential browser actions. Mitigation: add parallelism (2-3 concurrent Playwright pages) only if user-facing wait time exceeds 2 hours.
2. **Second bottleneck:** Batch API clustering latency. Hard floor of ~5 minutes, typical 15-30 minutes. Not addressable — just show a progress indicator and let it run.

---

## Build Order (Phase Implications)

Components have hard dependencies. Build in this order:

```
Phase 1 (Foundation):
  ZIP extraction + JSON streaming parser (Rust) → SQLite store
  Must come first — everything else reads from the store.

Phase 2 (Pipeline UI):
  Tauri commands exposing parse pipeline
  Progress streaming via Tauri Channel
  Drop Zone + ParseProgress React components
  End state: user can drop a ZIP and see parsed conversation count.

Phase 3 (Clustering):
  Anthropic Batch API client (Rust)
  Chunker + batch request builder
  Polling loop + result writer
  Basic Cluster Preview UI (read-only)
  End state: user can see proposed project groupings.

Phase 4 (Preview Editing):
  Editable ClusterPreview (rename projects, merge, exclude)
  ProjectManifest persistence in SQLite
  End state: user confirms manifest before any upload.

Phase 5 (Upload — highest risk):
  Playwright sidecar (Node.js)
  Sidecar IPC protocol
  Upload Driver (Rust orchestration)
  Upload Progress UI
  Session key management (Keychain)
  End state: full migration works end-to-end.
  NOTE: This phase requires research into current claude.ai session auth patterns
  before implementation. The unofficial endpoints may differ from what claude-pyrojects
  uses today.

Phase 6 (Polish):
  Error recovery, retry logic
  Images/attachment handling
  Custom instructions per project
  App signing and packaging
```

---

## Sources

- **Anthropic Admin API docs** (confirmed no Projects endpoint): https://platform.claude.com/docs/en/api/administration-api — HIGH confidence
- **Anthropic Files API docs** (confirmed separate from Projects): https://platform.claude.com/docs/en/build-with-claude/files — HIGH confidence
- **Anthropic Message Batches API docs** (confirmed batch limits, pricing, latency): https://platform.claude.com/docs/en/build-with-claude/batch-processing — HIGH confidence
- **claude-pyrojects** (unofficial Projects automation, session key auth confirmed): https://github.com/hcevikdotpy/claude-pyrojects — MEDIUM confidence (unofficial, may be stale)
- **stream-json** (Node.js streaming JSON parser): https://github.com/uhop/stream-json — HIGH confidence
- **unzipper** (Node.js streaming ZIP): https://github.com/ZJONSSON/node-unzipper — HIGH confidence
- **Tauri 2.0 IPC + FS docs** (confirmed raw IPC payloads, FS plugin): https://v2.tauri.app/concept/inter-process-communication/ — HIGH confidence
- **Tauri vs Electron 2026 comparison**: https://blog.nishikanta.in/tauri-vs-electron-the-complete-developers-guide-2026 — MEDIUM confidence (blog, aligns with official Tauri benchmarks)
- **ChatGPT conversations.json schema**: https://community.openai.com/t/questions-about-the-json-structures-in-the-exported-conversations-json/954762 — MEDIUM confidence (community docs, structure stable but undocumented officially)

---

*Architecture research for: ChatGPT → Claude migration Mac desktop app*
*Researched: 2026-02-28*
