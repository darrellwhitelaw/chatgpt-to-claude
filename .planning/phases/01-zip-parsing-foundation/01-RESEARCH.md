# Phase 1: ZIP Parsing Foundation - Research

**Researched:** 2026-02-28
**Domain:** Tauri 2.x scaffold + Rust ZIP/JSON streaming + SQLite storage + React drag-drop UI
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Visual Theme & Identity**
- Always light — white/minimal, not dark
- Reference: context-pack.com aesthetic (spacious, clean, minimal copy) but light not dark
- Purple accent color is NOT used — stay minimal white/neutral
- Feels like a focused utility tool, not a workspace

**Window Behavior**
- Fixed compact window, approximately 600×500px
- Not resizable — utility app feel
- Centers on launch

**Drop Zone Design**
- Dedicated centered drop zone with dashed border and icon
- NOT full-window drop target
- Minimal copy: one line of instruction ("Drop your ChatGPT export here")
- Nothing else shown on the empty state — no step indicators, no marketing

**File Picker**
- "Browse" / "select file" link sits inside the drop zone as secondary affordance
- Drop zone is primary; file picker is the "or" fallback
- No separate button below the zone

**Progress States**
- Simple stage labels + spinner — no numbers, no byte counts
- Human-readable stages: e.g. "Extracting ZIP…", "Parsing conversations…", "Building index…"
- Non-technical — zero jargon

**Completion State**
- Summary card: "Found [N] conversations ([year range])"
- Single prominent Continue button
- No list of conversation titles — just the aggregate summary

**Error Handling**
- Inline error message — shown within or adjacent to the drop zone
- Drop zone resets immediately so user can try again without restarting
- No dedicated error screen

### Claude's Discretion
- Exact dashed border styling (dash pattern, corner radius)
- Drop zone icon/illustration
- Exact padding, typography scale
- Stage label wording (can refine as long as it's non-technical and human-readable)
- Transition animations between states

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| IMP-01 | User can drag-and-drop a ChatGPT export ZIP onto the app window to begin migration | Tauri 2 `getCurrentWindow().onDragDropEvent()` — drop event yields `paths[]`; must listen to `tauri://drag-drop` event type `'drop'` |
| IMP-02 | User can use a file picker button as an alternative to drag-and-drop | `@tauri-apps/plugin-dialog` `open()` with `filters: [{ name: 'ZIP', extensions: ['zip'] }]` |
| IMP-03 | App displays a progress bar with status text during ZIP extraction and JSON streaming | Tauri 2 `Channel<T>` API — Rust backend sends typed progress events to frontend via channel; no polling needed |
| IMP-04 | App streams and parses conversations.json without loading the entire file into memory | Rust `zip` crate 8.x `ZipArchive::by_name()` returns `Read`-implementing `ZipFile`; wrap in `BufReader`; feed to `serde_json::Deserializer::from_reader().into_iter::<T>()` streaming deserializer |
| IMP-05 | App handles null, missing, or unexpected fields in conversations.json gracefully | All fields typed as `Option<T>` in Rust structs; `serde(default)` and `skip_serializing_if`; unknown types logged, not panicked |
| IMP-06 | App correctly reconstructs conversation message order by traversing the node-graph structure | Walk `current_node` → `parent` chain backward from `current_node`, collect messages, reverse to get chronological order; unit-tested with branched fixture |
</phase_requirements>

---

## Summary

Phase 1 establishes the Tauri 2 project scaffold and implements the entire streaming import pipeline: ZIP streaming in Rust, serde_json streaming array deserialization, SQLite storage, Tauri IPC Channel progress events, and the React drag-and-drop UI. The primary technical risk is the conversations.json tree-traversal requirement (IMP-06) — the format uses a node-graph with `mapping` keyed by message ID and requires walking from `current_node` backward through parent references, not iterating `mapping` values directly. This must be unit-tested with a branched conversation fixture before any downstream work.

The second technical risk is the serde_json streaming array pattern. `conversations.json` is a top-level JSON array, and `serde_json::StreamDeserializer` is designed for sequences of top-level values, not array elements. The correct approach is `Deserializer::from_reader().into_iter::<ConversationExport>()`, which wraps array element deserialization through serde's sequence visitor and processes one element at a time without buffering. A `BufReader` wrapper is required because serde_json does not buffer `Read` inputs.

The React frontend is intentionally thin. All business logic runs in Rust. The UI drives three state transitions: idle drop zone → progress display → completion summary card. Tauri's `Channel` API streams typed progress events from Rust to TypeScript with message-ordering guarantees, which is the correct mechanism for this use case over the older event system.

**Primary recommendation:** Scaffold with `pnpm create tauri-app` using the React TypeScript template. Implement the Rust pipeline in order: ZIP open → by_name lookup → BufReader wrapping → serde_json streaming → defensive normalization → SQLite insert per conversation → progress event emission. Wire the Tauri command to accept a file path string and return via channel. Build tree-traversal with a unit test fixture before writing normalization code.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.4.x | Desktop shell — Rust backend + WebKit UI | Official; smallest binary; genuine macOS native feel; Rust for safe file I/O |
| React | 19.x | Frontend UI framework | Official Tauri template; team velocity; TypeScript-first |
| TypeScript | 5.x | Type safety across frontend + IPC bridge | Native to Tauri React template |
| Vite | 6.x (template may ship 7.x) | Frontend bundler | Official Tauri integration; HMR for dev |
| `zip` (Rust crate) | 8.x | ZIP archive reading | Actively maintained; ZIP64 support; `by_name()` returns `Read`-implementing `ZipFile` |
| `serde` + `serde_json` | 1.x | JSON deserialization of conversations.json | Standard Rust JSON; `Deserializer::from_reader().into_iter()` for streaming arrays |
| `rusqlite` | 0.32.x | SQLite in Rust | Direct, no-magic; established Tauri pattern; straightforward connection management |
| `@tauri-apps/api` | 2.x | TypeScript IPC bridge | Official; `invoke()` + `Channel<T>` for progress streaming |
| `@tauri-apps/plugin-dialog` | 2.x | Native file picker | Official Tauri plugin; open() with zip extension filter |
| Tailwind CSS | 4.x | Utility styling | shadcn/ui companion; v4 is now standard |
| shadcn/ui | latest | UI component primitives | Copy-into-project model; Radix UI primitives; works with Tauri + Vite |
| Zustand | 4.x | App state | Lightweight; no boilerplate; one-shot migration state |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-specta | 2.x (rc) | Type-safe Rust↔TS IPC bindings | Optional but recommended — eliminates runtime errors at IPC boundary; generates `bindings.ts` |
| `@tauri-apps/api/mocks` | 2.x | `mockIPC()` for Vitest tests | Frontend tests that call Tauri commands without running the Rust backend |
| vitest | 2.x | Unit testing | Same config as Vite; mock Tauri IPC; test tree traversal logic in isolation |
| `lucide-react` | latest | Icons | Ships with shadcn templates; SVG icon for drop zone |
| `pnpm` | 9.x | Package manager | Faster than npm; works with Tauri; Tauri template supports it |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `zip` crate | `rc-zip`, `stream-unzip`, `async_zip` | `zip` is the most stable/documented; `rc-zip` supports async but adds complexity; `stream-unzip` is for sequential streaming without central directory (riskier for finding a specific file by name) |
| `rusqlite` | `tauri-plugin-sql` (wraps sqlx), `tauri-plugin-rusqlite2` | Official `tauri-plugin-sql` uses sqlx and works but adds plugin complexity; direct `rusqlite` is simpler when all DB access stays in Rust backend |
| `tauri-specta` | Manual TypeScript types | tauri-specta generates types automatically and prevents drift; manual types work but require discipline |
| Zustand | React Context / useState | Zustand is simpler for cross-component state without provider nesting; no overkill for this phase |

**Installation:**
```bash
# Scaffold (interactive — choose React, TypeScript, pnpm)
pnpm create tauri-app chatgpt-to-claude

# Frontend dependencies
pnpm add @tauri-apps/plugin-dialog zustand lucide-react
pnpm add -D vitest @vitest/ui

# shadcn/ui init (after Tailwind v4 is configured)
pnpm dlx shadcn@latest init

# Rust dependencies (Cargo.toml)
# [dependencies]
# zip = "2"      ← NOTE: verify latest 8.x on crates.io at build time
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# rusqlite = { version = "0.32", features = ["bundled"] }
# tauri-specta = { version = "2", features = ["javascript", "typescript"] }  # optional
```

---

## Architecture Patterns

### Recommended Project Structure

```
src-tauri/
├── src/
│   ├── lib.rs                   # Tauri app entry, command registration
│   ├── commands/
│   │   └── ingest.rs            # Tauri command: parse_zip(path, on_event)
│   ├── pipeline/
│   │   ├── zip_reader.rs        # ZipArchive::new(File) + by_name()
│   │   ├── json_parser.rs       # serde_json streaming iterator
│   │   ├── traversal.rs         # current_node→parent chain walker
│   │   └── normalizer.rs        # ConversationExport → ConversationRecord
│   └── store/
│       ├── db.rs                # rusqlite connection, init, migrations
│       └── schema.sql           # CREATE TABLE conversations
src/
├── components/
│   ├── DropZone.tsx             # Drag-drop + browse link UI
│   ├── ProgressView.tsx         # Stage label + spinner
│   └── SummaryCard.tsx          # "Found N conversations (year range)" + Continue
├── hooks/
│   └── useIngest.ts             # Drives ingest command via invoke + Channel
├── lib/
│   └── bindings.ts              # Generated by tauri-specta (or manual types)
└── App.tsx                      # State machine: idle → parsing → complete
```

### Pattern 1: Tauri Command with Progress Channel

**What:** A long-running Rust command that streams typed progress events to the React frontend using Tauri 2's `Channel<T>` API.

**When to use:** Any operation that takes more than ~200ms and needs to report intermediate state. Required for ZIP parsing (IMP-03).

**Rust side:**
```rust
// Source: https://v2.tauri.app/develop/calling-frontend/
use tauri::ipc::Channel;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum IngestEvent {
    Started,
    ExtractingZip,
    ParsingConversations { processed: u32 },
    BuildingIndex,
    Complete { total: u32, earliest_year: i32, latest_year: i32 },
    Error { message: String },
}

#[tauri::command]
pub async fn parse_zip(
    path: String,
    on_event: Channel<IngestEvent>,
) -> Result<(), String> {
    on_event.send(IngestEvent::Started).map_err(|e| e.to_string())?;
    // ... pipeline ...
    Ok(())
}
```

**TypeScript side:**
```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/
import { invoke, Channel } from '@tauri-apps/api/core';

const onEvent = new Channel<IngestEvent>();
onEvent.onmessage = (msg) => {
  if (msg.event === 'parsingConversations') {
    setProcessed(msg.data.processed);
  } else if (msg.event === 'complete') {
    setSummary(msg.data);
  }
};

await invoke('parse_zip', { path: filePath, onEvent });
```

### Pattern 2: Streaming JSON Array with serde_json

**What:** Reading a large top-level JSON array element by element using serde_json's Deserializer with `into_iter()`. This avoids loading the full array into memory.

**When to use:** Always — `conversations.json` is a top-level array that can be 600MB+.

**Critical note:** `serde_json::StreamDeserializer` is for sequences of *multiple top-level values*. `conversations.json` is a *single top-level array*. The correct approach is `into_iter::<T>()` on the Deserializer, which processes array elements through serde's sequence visitor.

```rust
// Source: serde_json docs + serde.rs/stream-array.html
use std::fs::File;
use std::io::BufReader;
use serde_json::Deserializer;

pub fn stream_conversations<P: AsRef<std::path::Path>>(
    path: P,
) -> impl Iterator<Item = Result<ConversationExport, serde_json::Error>> {
    let file = File::open(path).expect("cannot open conversations.json");
    let reader = BufReader::new(file); // serde_json does not buffer — always wrap with BufReader
    Deserializer::from_reader(reader).into_iter::<ConversationExport>()
}
```

**For ZIP entry (IMP-04 — streaming directly from ZIP without extracting to disk):**
```rust
// zip crate: ZipFile implements Read, so it chains to BufReader directly
let file = File::open(&zip_path)?;
let mut archive = zip::ZipArchive::new(file)?;
let entry = archive.by_name("conversations.json")?;
let reader = BufReader::new(entry);
let stream = Deserializer::from_reader(reader).into_iter::<ConversationExport>();

for result in stream {
    let conv = result.map_err(|e| e.to_string())?;
    let record = normalize(conv);
    db.insert_conversation(&record)?;
    on_event.send(IngestEvent::ParsingConversations { processed: count })?;
    count += 1;
}
```

### Pattern 3: Tree Traversal for Message Order (IMP-06)

**What:** The `conversations.json` `mapping` field is a node-graph, not a flat list. The only correct way to reconstruct a conversation's message order is to start at `current_node` and follow `parent` references backward, then reverse.

**When to use:** Always — any other approach silently produces wrong output.

```rust
// Pseudocode — implement in traversal.rs
pub fn linearize_messages(
    mapping: &HashMap<String, MessageNode>,
    current_node: &str,
) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut node_id = Some(current_node.to_string());

    while let Some(id) = node_id {
        if let Some(node) = mapping.get(&id) {
            if let Some(ref msg) = node.message {
                if should_include_message(msg) {
                    messages.push(msg.clone());
                }
            }
            node_id = node.parent.clone();
        } else {
            break; // missing parent node — handle gracefully
        }
    }

    messages.reverse(); // was built root→leaf, need leaf→root reversed
    messages
}

fn should_include_message(msg: &Message) -> bool {
    // Only include user and assistant messages; skip system/tool/memory
    matches!(msg.author.role.as_str(), "user" | "assistant")
        && msg.content.is_some()
        && !is_empty_content(msg)
}
```

### Pattern 4: Defensive Parsing for conversations.json (IMP-05)

**What:** Every field that is nullable or undocumented in conversations.json must use `Option<T>` and `#[serde(default)]`.

**When to use:** Always — the format changes without documentation. March 2025 attachment format change is a confirmed example.

```rust
#[derive(Debug, serde::Deserialize)]
pub struct ConversationExport {
    pub id: String,
    pub title: Option<String>,         // can be null/missing
    pub create_time: Option<f64>,      // unix timestamp; can be null
    pub update_time: Option<f64>,
    pub mapping: HashMap<String, MessageNode>,
    pub current_node: Option<String>,  // absent in some older exports
}

#[derive(Debug, serde::Deserialize)]
pub struct MessageNode {
    pub id: String,
    pub parent: Option<String>,        // null for root node
    pub children: Vec<String>,
    pub message: Option<Message>,      // null for tree structure nodes
}

#[derive(Debug, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub author: Author,
    pub create_time: Option<f64>,      // null in many messages
    pub content: Option<Content>,      // null for tool-call nodes
    #[serde(default)]
    pub metadata: serde_json::Value,   // fully open — absorbs unknown fields
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    pub content_type: String,          // "text", "multimodal_text", "code", etc.
    #[serde(default)]
    pub parts: Vec<serde_json::Value>, // can be strings, image objects, or empty
}
```

### Pattern 5: SQLite Initialization in Tauri Setup

**What:** Initialize the SQLite database in the Tauri `setup()` closure, store the connection in managed state, and expose it to commands.

```rust
// lib.rs
use rusqlite::Connection;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = app.path()
                .app_data_dir()?
                .join("conversations.db");
            let conn = Connection::open(&db_path)?;
            init_schema(&conn)?;
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![parse_zip])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT,
            created_at INTEGER,
            message_count INTEGER NOT NULL DEFAULT 0,
            has_images INTEGER NOT NULL DEFAULT 0,
            has_code INTEGER NOT NULL DEFAULT 0,
            token_estimate INTEGER NOT NULL DEFAULT 0,
            full_text TEXT NOT NULL DEFAULT '',
            cluster_id TEXT,
            project_name TEXT
        );
    ")
}
```

### Pattern 6: Window Configuration (tauri.conf.json)

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "ChatGPT to Claude",
        "width": 600,
        "height": 500,
        "resizable": false,
        "center": true,
        "minWidth": 600,
        "maxWidth": 600,
        "minHeight": 500,
        "maxHeight": 500
      }
    ]
  }
}
```

**macOS note:** On macOS the height includes the title bar (outer measurement). If content is visually cut off, increase height by ~28px (standard macOS title bar height) to compensate.

### Anti-Patterns to Avoid

- **Full ZIP extraction to disk before parsing:** Doubles disk requirements; 5GB export = 5GB temp. Use `zip::ZipArchive::by_name()` to access `conversations.json` entry directly, then stream from it.
- **`JSON.parse` / `from_reader` without streaming:** Loading entire `conversations.json` into memory causes OOM on exports >200MB. Always stream.
- **Iterating `mapping.values()` to get messages:** Produces random order and includes all branches, not the branch the user saw. Always walk `current_node` → parent chain.
- **Panicking on null fields:** `conversations.json` has had format changes without documentation. Every `unwrap()` on a JSON field is a future crash. Use `Option<T>` everywhere.
- **Loading the full message text for the progress counter:** The progress counter only needs a count increment per conversation, not the full text. Emit `IngestEvent::ParsingConversations { processed }` without reading the full text into the event.
- **Calling `invoke_handler` twice:** Tauri only uses the last call. Pass all commands to a single `generate_handler![]` macro call.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ZIP reading with entry-by-name lookup | Custom ZIP byte parser | `zip` crate 8.x | ZIP64, Deflate/Bzip2 support, tested, `ZipFile` implements `Read` |
| JSON streaming with error recovery | Custom token-by-token reader | `serde_json` + serde derive | Already handles every edge case; `into_iter()` propagates deserialization errors per item |
| Progress event delivery to frontend | Custom WebSocket or polling | Tauri `Channel<T>` | Ordered, typed, built into Tauri 2; correct tool for this exact use case |
| File picker dialog | Custom `<input type="file">` | `@tauri-apps/plugin-dialog` | Native macOS file picker; proper sandbox permissions; extension filtering |
| Drag-and-drop file path | HTML5 `dragover`/`drop` | `getCurrentWindow().onDragDropEvent()` | HTML5 drag API doesn't give file paths in Tauri; only Tauri's native event gives real paths |
| SQLite connection management | Raw file I/O | `rusqlite` with `Mutex<Connection>` in Tauri managed state | Thread-safe access, proper transactions, in-memory option for tests |
| Node-graph traversal test fixtures | Testing against real export files | Inline `HashMap` fixture in unit test | Deterministic; no large file in repo; covers branches explicitly |

**Key insight:** The ZIP/JSON streaming pipeline involves enough edge cases (ZIP64, decompression codec, null JSON fields, Unicode normalization, tree branches) that every hand-rolled implementation will fail silently on edge-case exports from users with large histories.

---

## Common Pitfalls

### Pitfall 1: Tree Traversal — Silent Wrong Output
**What goes wrong:** Parser iterates `mapping.values()` or assumes `mapping` is ordered. Produces wrong message order, missing branches, or returns only the last alternative when user retried a response.

**Why it happens:** `mapping` is a keyed dict for a tree structure. The order of values is not conversation order. Only `current_node` + parent traversal is correct.

**How to avoid:** Write `traversal.rs` with explicit `current_node → parent` walk. Write a unit test with a `HashMap` fixture containing at least one branched conversation (two children from the same parent, with `current_node` pointing to one branch). Verify the correct branch messages are returned.

**Warning signs:** Message counts look right but message order is alphabetical or insertion-order. Conversations with "regenerate response" appear to have both responses.

### Pitfall 2: serde_json StreamDeserializer vs Array Streaming
**What goes wrong:** Using `StreamDeserializer` (designed for *multiple top-level JSON values* in a stream) on a file whose only content is a single JSON array. This loads the entire array into one deserialize call — the opposite of streaming.

**Why it happens:** The name "StreamDeserializer" suggests general-purpose streaming. It is not — it handles `{"a":1}{"b":2}` (NDJSON-like), not `[{"a":1},{"b":2}]`.

**How to avoid:** Use `Deserializer::from_reader(reader).into_iter::<ConversationExport>()` — this goes through serde's sequence visitor and yields one `ConversationExport` at a time from the array.

**Warning signs:** No memory pressure during parsing of a tiny test file, then OOM crash on a 600MB export.

### Pitfall 3: Missing BufReader Wrapping
**What goes wrong:** Passing `ZipFile` or `File` directly to serde_json without wrapping in `BufReader`. Performance degrades significantly because serde_json reads one byte at a time from unbuffered sources.

**Why it happens:** serde_json's `from_reader` accepts any `Read` — it works, but slowly. The official docs explicitly warn: "serde_json will not buffer the input."

**How to avoid:** Always `BufReader::new(entry)` before passing to the deserializer.

**Warning signs:** Parsing a 100MB file takes 30+ seconds. CPU usage is high but throughput is low.

### Pitfall 4: Tauri Drag-Drop vs HTML5 Drop API
**What goes wrong:** Component uses React `onDrop` / `onDragOver` event handlers expecting to receive `event.dataTransfer.files`. In Tauri, the native drag-drop event does not populate `dataTransfer` with local file paths — the HTML5 drop API returns empty or partial data.

**Why it happens:** Tauri intercepts OS-level file drops and routes them through its own event system. The HTML5 drag API works for web content dragged in, not local file system drops.

**How to avoid:** Use `getCurrentWindow().onDragDropEvent()` from `@tauri-apps/api/window`. Listen for `payload.type === 'drop'` to get `payload.paths` (array of absolute file system paths). The HTML5 `onDragOver` can still be used to style the drop target (preventing default to allow drop), but the actual path comes from the Tauri event.

**Warning signs:** Drop handler fires but `event.dataTransfer.files` is empty or contains a blob URL instead of a real file path.

### Pitfall 5: macOS Window Height Includes Title Bar
**What goes wrong:** Setting `height: 500` in `tauri.conf.json` on macOS produces a window where the content area is ~472px tall, not 500px. CSS layout designed for 500px is visually clipped.

**Why it happens:** On macOS, `height` is the *outer* size (including the ~28px title bar). On Windows/Linux it's the inner size.

**How to avoid:** Use `height: 528` in `tauri.conf.json` for macOS if targeting 500px of usable content, or design CSS to fill the available height responsively so the exact pixel count doesn't matter.

### Pitfall 6: `invoke_handler` Called Multiple Times
**What goes wrong:** Commands registered in two separate `.invoke_handler()` calls. Only the last call's commands are reachable from the frontend.

**Why it happens:** The builder pattern looks additive but is overwriting.

**How to avoid:** Pass all commands to one `tauri::generate_handler![cmd1, cmd2, cmd3]` call.

### Pitfall 7: Conversations.json Schema Instability
**What goes wrong:** Parser written against current schema breaks when a user drops an export from a different time period (pre-2023 format, post-March-2025 attachment format change, etc.).

**Why it happens:** OpenAI adds new message types with every product launch without versioning the export format.

**How to avoid:** Use `serde_json::Value` for `metadata` and `parts` element types. Log (don't panic) on unknown `content_type` values. Include exports from at least two different time periods in the test fixture suite.

---

## Code Examples

Verified patterns from official sources:

### Drag-and-Drop File Path (TypeScript)
```typescript
// Source: https://v2.tauri.app/reference/javascript/api/namespacewindow/
import { getCurrentWindow } from '@tauri-apps/api/window';

const unlisten = await getCurrentWindow().onDragDropEvent((event) => {
  if (event.payload.type === 'enter') {
    setIsDragging(true);
  } else if (event.payload.type === 'drop') {
    setIsDragging(false);
    const paths = event.payload.paths; // string[] of absolute file paths
    const zipPath = paths.find(p => p.endsWith('.zip'));
    if (zipPath) {
      startIngest(zipPath);
    }
  } else if (event.payload.type === 'leave') {
    setIsDragging(false);
  }
});

// Clean up on component unmount
useEffect(() => { return () => { unlisten(); }; }, []);
```

### File Picker (TypeScript)
```typescript
// Source: https://v2.tauri.app/plugin/dialog/
import { open } from '@tauri-apps/plugin-dialog';

async function browseForFile() {
  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'ChatGPT Export', extensions: ['zip'] }],
  });
  if (path) {
    startIngest(path as string);
  }
}
```

### Channel-Based Progress (TypeScript)
```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/
import { invoke, Channel } from '@tauri-apps/api/core';

async function startIngest(path: string) {
  const onEvent = new Channel<IngestEvent>();
  onEvent.onmessage = (msg) => {
    switch (msg.event) {
      case 'extractingZip':      setStage('Extracting ZIP…'); break;
      case 'parsingConversations':
        setStage('Parsing conversations…');
        setProcessed(msg.data.processed);
        break;
      case 'buildingIndex':      setStage('Building index…'); break;
      case 'complete':           setSummary(msg.data); setStage('done'); break;
      case 'error':              setError(msg.data.message); break;
    }
  };
  await invoke('parse_zip', { path, onEvent });
}
```

### Tauri Command Registration (Rust)
```rust
// Source: https://v2.tauri.app/develop/calling-rust/
// lib.rs
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // db init here
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::ingest::parse_zip,
            // future commands added here
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Vitest Mock for Tauri Commands (TypeScript)
```typescript
// Source: https://v2.tauri.app/develop/tests/mocking/
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { render, screen } from '@testing-library/react';
import { DropZone } from '../components/DropZone';

beforeEach(() => {
  mockIPC((cmd, args) => {
    if (cmd === 'parse_zip') {
      // simulate progress events if needed
      return Promise.resolve();
    }
  });
});

afterEach(() => clearMocks());
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 event system for streaming | Tauri v2 `Channel<T>` API | Tauri 2.0 stable (Oct 2024) | Channel is ordered, typed, faster — use it instead of `emit()` for progress |
| `adm-zip` (in-memory ZIP) | `zip` crate 8.x or `unzipper` npm | Ongoing — adm-zip is still maintained but wrong for large files | Memory-safe streaming ZIP is now standard; no excuse to load full archive |
| Manual TypeScript types for Tauri commands | `tauri-specta` generated bindings | 2024, stable for Tauri 2 | Type-safe IPC eliminates a class of runtime errors at the TS↔Rust boundary |
| Tauri Stronghold for credentials | `tauri-plugin-keyring` or direct `keyring` Rust crate | Tauri 2.x deprecation of Stronghold | Stronghold will be removed in Tauri v3; avoid it entirely for new projects |

**Deprecated/outdated:**
- `tauri::window::Builder::with_file_drop_handler()`: Tauri v1 pattern. In Tauri 2, use `onDragDropEvent()` from the JavaScript API instead.
- `adm-zip`: Still maintained but loads entire archive into memory — disqualified for this use case.
- Tauri Stronghold plugin: Will be removed in Tauri v3. Use `tauri-plugin-keyring` for Phase 2.

---

## Open Questions

1. **Safari's auto-unzip behavior (macOS)**
   - What we know: When a macOS user downloads a ZIP in Safari, Safari may automatically decompress it to a folder, leaving `export.zip (1)` or just a folder.
   - What's unclear: How common this is in practice; whether the app should handle folder drops as well as ZIP drops.
   - Recommendation: In Phase 1, support ZIP files only. Add a clear inline error for dropped folders: "Looks like Safari unzipped this automatically — find the .zip file in your Downloads folder." Revisit in Phase 5 polish.

2. **`tauri-specta` v2 stability for new projects**
   - What we know: Version is `2.0.0-rc.x` (release candidate). Works with Tauri 2. Actively developed.
   - What's unclear: Whether rc status introduces breaking changes during the Phase 1 build window.
   - Recommendation: Use tauri-specta. Its rc status is stable in practice (multiple production apps use it). If a breaking change occurs, the manual fallback is straightforward TypeScript types.

3. **rusqlite `bundled` feature vs system SQLite**
   - What we know: `rusqlite` with `features = ["bundled"]` compiles SQLite into the binary. Without it, links to system SQLite.
   - What's unclear: Whether bundled adds significant binary size impact for macOS distribution.
   - Recommendation: Use `bundled` for predictable behavior across macOS versions. Binary size overhead is ~1MB — negligible for a Tauri app already at ~10MB.

4. **conversations.json nested inside ZIP at different paths**
   - What we know: OpenAI exports typically have `conversations.json` at the ZIP root. Some reports suggest sub-directory paths in certain export versions.
   - What's unclear: Whether `archive.by_name("conversations.json")` always works or needs a fallback search.
   - Recommendation: Try `by_name("conversations.json")` first. On `NotFound`, iterate entries looking for a file named `conversations.json` at any depth using `entry.name().ends_with("/conversations.json")`.

---

## Sources

### Primary (HIGH confidence)
- Tauri 2 official — Create project: https://v2.tauri.app/start/create-project/
- Tauri 2 official — Calling frontend / Channel API: https://v2.tauri.app/develop/calling-frontend/
- Tauri 2 official — Calling Rust / command patterns: https://v2.tauri.app/develop/calling-rust/
- Tauri 2 official — Dialog plugin: https://v2.tauri.app/plugin/dialog/
- Tauri 2 official — Mocking IPC in tests: https://v2.tauri.app/develop/tests/mocking/
- Tauri 2 official — Window customization / config: https://v2.tauri.app/learn/window-customization/
- Tauri 2 official — Configuration reference: https://v2.tauri.app/reference/config/
- Tauri 2 official — Window namespace (onDragDropEvent): https://v2.tauri.app/reference/javascript/api/namespacewindow/
- zip crate docs (v8.x): https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html
- serde_json StreamDeserializer docs: https://docs.rs/serde_json/latest/serde_json/struct.StreamDeserializer.html
- serde streaming array pattern: https://serde.rs/stream-array.html
- tauri-specta v2 docs: https://specta.dev/docs/tauri-specta/v2

### Secondary (MEDIUM confidence)
- Tauri drag-drop issue (event name + payload structure confirmed): https://github.com/tauri-apps/tauri/issues/9830
- serde_json issue #404 — streaming top-level array discussion: https://github.com/serde-rs/json/issues/404
- rusqlite + Tauri setup pattern: https://blog.moonguard.dev/how-to-use-local-sqlite-database-with-tauri
- tauri-plugin-rusqlite2 (alternative approach): https://crates.io/crates/tauri-plugin-rusqlite2
- Dannysmith tauri-template (production-ready scaffold reference): https://github.com/dannysmith/tauri-template

### Tertiary (LOW confidence)
- macOS window height outer-vs-inner behavior: mentioned in multiple GitHub issues (tauri-apps/tauri#4226, #6333) — verified as real behavior, exact pixel offset varies by macOS version

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All libraries confirmed from official Tauri 2 docs, crates.io, and official patterns
- Architecture: HIGH — Patterns verified from official Tauri 2 docs; serde_json streaming confirmed from official serde docs
- Pitfalls: HIGH — Drag-drop API confirmed from Tauri source; serde_json array streaming constraint confirmed from official issue tracker; tree traversal requirement confirmed from project-level research (ARCHITECTURE.md, PITFALLS.md)
- Code examples: HIGH — All examples derived from official documentation sources

**Research date:** 2026-02-28
**Valid until:** 2026-03-28 (Tauri moves fast; verify plugin versions before building; tauri-specta rc version may bump)
