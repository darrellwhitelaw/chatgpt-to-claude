# Phase 2: API Key + AI Clustering - Research

**Researched:** 2026-02-28
**Domain:** macOS Keychain, Anthropic Batch API, token counting, Tauri v2 async commands, SQLite schema extension
**Confidence:** HIGH (all critical paths verified against official sources)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**API Key Entry Screen**
- Appears after the summary card — user sees "Found N conversations", clicks Continue, then hits key entry if no key is stored
- Minimal UI: password-style input + label + Continue button only. No upfront cost warning on this screen
- User can change the key from the summary card screen (small "change key" link)
- Invalid/rejected key shows an inline error under the input field: "Invalid API key — check console.anthropic.com". Stay on same screen, no modal

**Cost Estimate Screen**
- Shows: token count + dollar estimate — e.g. "~2.4M tokens · estimated $1.20"
- Two actions: Proceed / Cancel. Cancel returns to the summary card
- Model: claude-haiku-3-5 for everything (clustering + summaries). Cheapest, fast batch processing, sufficient quality
- If estimated cost exceeds a threshold (e.g. $5): show a warning callout — "This is higher than typical — your export is large." Let user decide without alarm

**Clustering Progress UI**
- Spinner + stage label — same pattern as Phase 1 ZIP parsing. e.g. "Clustering conversations...", "Generating summaries..."
- User must stay in the app — no background/resume support in v1. Clustering should complete within ~5 minutes for typical exports
- On batch failure: error screen with "Try again" — returns user to cost estimate screen to resubmit
- Polling: every 5 seconds until the batch completes

**Conversation Summaries**
- Each summary contains: key decisions + conclusions + main topic
- Length: 3–5 sentences — skimmable but contextually useful
- Custom instructions extraction: system prompts + explicit user instruction patterns (e.g. "always respond in bullet points") found in the conversation, not just role=system messages
- Cluster count: dynamic — let Claude decide 5–20 clusters based on actual content distribution. No fixed target, no user input required

### Claude's Discretion
- Exact Keychain integration library/approach (tauri-plugin-keychain or native)
- Prompt design for clustering and summarization
- Exact threshold for "high cost" warning
- Token estimation approach (pre-flight vs exact count from API response)

### Deferred Ideas (OUT OF SCOPE)
- Background batch processing with resume on reopen — Phase 2+ or v2
- User-configurable cluster target count — noted in REQUIREMENTS.md as CLUST-01 (v2)
- Model selection UI — keep single model for v1, may revisit in v2
- Processing a subset of conversations (cap by count/date) — v2 scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SEC-01 | User can enter Anthropic API key; stored in macOS Keychain, never written to disk | keyring crate v3.6.3 with apple-native feature; Entry::new + set_password/get_password pattern documented |
| SEC-02 | App displays API key entry UI on first launch if no key is stored | Keychain `get_password` returns `Err(NoEntry)` on first launch; React state gating on AppPhase `awaiting-key` |
| AI-01 | App uses Anthropic Message Batches API to cluster conversations into named project groups | Batch API fully documented: POST /v1/messages/batches, 100K req limit, 256MB limit, JSONL results via results_url |
| AI-02 | App shows estimated API cost (tokens + approximate $) before clustering batch is submitted | /v1/messages/count_tokens endpoint confirmed; free to use; returns `{input_tokens: N}`; haiku-3-5 batch price: $0.40/$2.00 per MTok |
| AI-03 | App generates AI summary (key decisions, conclusions, context) for each conversation | Included as params in same batch as clustering; custom_id pattern maps result back to conversation_id |
| AI-04 | App extracts custom instructions and system prompts from ChatGPT conversations | Instruction extraction prompt in same batch pass; results stored as TEXT in new `instructions` column |
</phase_requirements>

## Summary

Phase 2 introduces three new technical surfaces: macOS Keychain for secret storage, the Anthropic Message Batches API for async AI processing, and schema extensions to SQLite for storing cluster assignments, summaries, and extracted instructions. All three surfaces have verified, stable APIs.

The macOS Keychain integration is best implemented via the `keyring` crate (v3.6.3, `apple-native` feature) called directly from Rust Tauri commands — no Tauri plugin required. The plugin options (`tauri-plugin-keyring`, `tauri-plugin-keychain`) are community-maintained with low adoption and one has a failing docs.rs build. Using the `keyring` crate directly via a Tauri command gives us full control and no plugin permission surface. The keyring crate handles the macOS Keychain Services framework transparently.

For AI processing, the Anthropic Message Batches API supports up to 100,000 requests per batch and returns 50% cost savings over standard pricing. The critical architectural insight is that both clustering AND summary generation AND instruction extraction should be submitted in a **single batch submission** — one `custom_id` per conversation, with a combined prompt that returns structured JSON. Polling at 5-second intervals matches the user decision and is safe (most batches complete within 1 hour). Token counting uses the dedicated `/v1/messages/count_tokens` endpoint (free, synchronous, no side effects) to produce the pre-flight cost estimate.

The existing SQLite schema already has `cluster_id TEXT` and `project_name TEXT` columns on the `conversations` table. Phase 2 needs to add `summary TEXT`, `instructions TEXT`, and `cluster_label TEXT` columns via ALTER TABLE migration in the schema init. The `full_text` column already exists and is the input for all AI prompts.

**Primary recommendation:** Use `keyring` crate directly (not a plugin), one combined batch per run with structured JSON output, and the `/v1/messages/count_tokens` endpoint for cost estimation. Add `reqwest` with `json` + `rustls-tls` features to Cargo.toml and use `tauri::async_runtime::spawn` (not `tokio::spawn`) for all background HTTP work.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `keyring` | 3.6.3 | macOS Keychain read/write | Official cross-platform crate; wraps Keychain Services on macOS; actively maintained; used by tauri-plugin-keyring internally |
| `reqwest` | 0.12.x | Async HTTP client for Anthropic API calls | De facto standard Rust HTTP client; re-exported by Tauri's HTTP plugin; `json` feature handles serde automatically |
| `serde` + `serde_json` | 1.x | Serialize/deserialize API request/response bodies | Already in Cargo.toml; required for structured batch request building and JSONL result parsing |
| `rusqlite` | 0.32 (bundled) | SQLite writes for cluster/summary/instruction results | Already in Cargo.toml; same connection pattern as Phase 1 |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri::async_runtime` | (bundled with tauri 2) | Safe async spawning within Tauri | Use `tauri::async_runtime::spawn` instead of `tokio::spawn` — Tauri v2 has known panic with raw `tokio::spawn` in some contexts |
| `tokio` | (via tauri) | Async runtime | Do NOT add separately; use tauri's re-export |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `keyring` crate directly | `tauri-plugin-keyring` (HuakunShen) | Plugin wraps the same crate; adds JS API surface we don't need; 14 stars, 1 contributor; adds capability permission overhead |
| `keyring` crate directly | `tauri-plugin-keychain` (lindongchen) | Tauri v2 compatible but has failing docs.rs build (v2.0.2); lower trust |
| `keyring` crate directly | Tauri Stronghold | Overkill — Stronghold is an encrypted secret vault; adds IOTA dependency; keyring is sufficient for a single API key |
| `/v1/messages/count_tokens` | Client-side char-count estimate (÷4) | Already done in Phase 1 as `token_estimate`; sufficient for rough display but off-API estimate; count_tokens is free and exact |

**Installation (Cargo.toml additions):**
```toml
keyring = { version = "3", features = ["apple-native"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

**No npm additions needed** — all API calls go through Rust commands, not JS fetch.

## Architecture Patterns

### Recommended Project Structure

New files to add in Phase 2:

```
src-tauri/src/
├── commands/
│   ├── ingest.rs          # Existing — Phase 1
│   ├── keychain.rs        # NEW — get/set/delete API key via keyring crate
│   └── cluster.rs         # NEW — cost estimate, submit batch, poll batch
├── ai/
│   ├── mod.rs             # NEW — pub mod batch; pub mod prompts
│   ├── batch.rs           # NEW — Anthropic API types, HTTP calls (reqwest)
│   └── prompts.rs         # NEW — prompt templates for clustering/summary/instructions
├── store/
│   ├── db.rs              # Extend — add update_cluster_result(), migration
│   └── schema.sql         # Extend — ALTER TABLE for summary/instructions/cluster_label

src/ (React/TS)
├── screens/
│   ├── ApiKeyScreen.tsx   # NEW — password input, inline error, continue button
│   └── CostScreen.tsx     # NEW — token count, dollar estimate, Proceed/Cancel
├── hooks/
│   ├── useKeychain.ts     # NEW — invoke keychain commands
│   └── useCluster.ts      # NEW — invoke cluster commands, Channel-based progress
└── store/
    └── appStore.ts        # Extend — add AppPhase variants
```

### Pattern 1: AppPhase Flow Extension

**What:** Add new AppPhase variants to the Zustand store to gate screen rendering.
**When to use:** Every new screen corresponds to a new phase value.

```typescript
// src/store/appStore.ts — extend existing AppPhase union
export type AppPhase =
  | 'idle'
  | 'parsing'
  | 'complete'           // summary card shown (Phase 1 output)
  | 'awaiting-key'       // no key in Keychain — show ApiKeyScreen
  | 'key-stored'         // key confirmed valid — show CostScreen
  | 'cost-ready'         // token count fetched — show Proceed/Cancel
  | 'clustering'         // batch submitted, polling active
  | 'clustering-complete' // batch done, SQLite written
  | 'error';

// New fields to add to AppState interface:
interface AppState {
  // ... existing fields ...
  tokenEstimate: number | null;
  costEstimateUsd: number | null;
  batchId: string | null;
  clusterError: string | null;
}
```

### Pattern 2: Keychain Tauri Commands

**What:** Three thin Tauri commands wrapping the `keyring` crate. No JS-side state — just invoke/error.
**When to use:** Any time we touch the macOS Keychain.

```rust
// src-tauri/src/commands/keychain.rs
// Source: https://docs.rs/keyring/3.6.3/keyring/

use keyring::Entry;

const SERVICE: &str = "com.darrellwhitelaw.chatgpt-to-claude";
const USER: &str = "anthropic-api-key";

#[tauri::command]
pub fn get_api_key() -> Result<String, String> {
    let entry = Entry::new(SERVICE, USER).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, USER).map_err(|e| e.to_string())?;
    entry.set_password(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE, USER).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())
}
```

**First-launch detection:** `get_api_key()` returns `Err("No matching entry found in secure storage")` when no key exists. React hook checks `phase === 'complete'` → calls `invoke('get_api_key')` → on error, transitions to `awaiting-key`.

### Pattern 3: Token Count Pre-flight

**What:** Call `/v1/messages/count_tokens` with the full batch payload shape (system prompt + conversation full_text sample or aggregate) to get exact token count before submission.
**When to use:** After user enters API key, before showing the cost screen.

**Strategy for token counting:** The count_tokens endpoint accepts the same shape as a Messages request. For cost estimation, send ONE representative count_tokens call with the clustering system prompt and all concatenated `full_text` values. This gives the input token count for the entire batch.

```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/token-counting
// POST https://api.anthropic.com/v1/messages/count_tokens
// Headers: x-api-key, anthropic-version: 2023-06-01, content-type: application/json
// Response: { "input_tokens": N }

// Cost formula (claude-haiku-3-5, batch pricing):
// input_cost  = (input_tokens  / 1_000_000) * 0.40
// output_cost = estimated_output_tokens / 1_000_000 * 2.00
// total = input_cost + output_cost
// Display: "~{input_tokens/1000}K tokens · estimated ${total:.2}"
```

**Token estimate already in DB:** `conversations.token_estimate` (char/4) gives a fast local sum for UI before the API call returns. Show local estimate immediately, replace with exact count when API responds.

### Pattern 4: Batch API Submit + Poll

**What:** One HTTP POST to create the batch, then repeated GET polls until `processing_status === "ended"`.
**When to use:** When user clicks Proceed on the cost screen.

```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/batch-processing
// Create: POST https://api.anthropic.com/v1/messages/batches
// Poll:   GET  https://api.anthropic.com/v1/messages/batches/{id}
// Results: GET {results_url}  (JSONL stream)

// Batch request structure per conversation:
// {
//   "custom_id": "{conversation_id}",
//   "params": {
//     "model": "claude-haiku-3-5-20241022",
//     "max_tokens": 1024,
//     "system": "{clustering + summary + instructions prompt}",
//     "messages": [{"role": "user", "content": "{full_text truncated to ~8000 chars}"}]
//   }
// }
```

**Channel-based progress to frontend:** Same `Channel<ClusterEvent>` pattern as `parse_zip`. Events: `BatchSubmitted { batch_id }`, `Polling { elapsed_secs }`, `Complete { assigned_count }`, `Error { message }`.

**Polling implementation:** Use `tauri::async_runtime::spawn` to run the poll loop off the main thread. Poll every 5 seconds (user decision). Emit `Polling` event each cycle so UI can show elapsed time.

### Pattern 5: Combined Prompt (One Batch, Three Outputs)

**What:** Each conversation gets ONE batch request that returns a structured JSON response containing cluster assignment, summary, and extracted instructions.
**Why:** Submitting three separate batches (cluster, summarize, extract) triples cost and complexity. One batch with a structured output prompt is cheaper and simpler.

```
System prompt structure:
"You are analyzing a ChatGPT conversation. Return ONLY a JSON object with these fields:
- cluster_label: string (2-4 words, a topical group name; choose from 5-20 possible labels
  based on the full corpus of conversations you'll see)
- summary: string (3-5 sentences covering key decisions, conclusions, and main topic)
- instructions: string | null (any custom instructions the user gave, e.g. 'always use
  bullet points', or null if none found)

User message will contain the conversation transcript."
```

**IMPORTANT:** Clustering requires global context (all conversations) to assign consistent labels. Two-pass approach:
1. **Pass 1 (cluster discovery):** Submit a single, separate (non-batch) request with a sample of conversation titles/snippets to get the 5-20 cluster label vocabulary.
2. **Pass 2 (batch):** Submit the full batch with the cluster vocabulary in the system prompt so all conversations get consistent labels.

This two-pass approach costs a small extra fee for Pass 1 but ensures cluster label consistency. Without it, batch requests process in isolation and may generate inconsistent or overlapping cluster names.

### Anti-Patterns to Avoid

- **Using `tokio::spawn` directly:** Tauri v2 has a known panic (`no reactor running`) when `tokio::spawn` is called from some contexts. Always use `tauri::async_runtime::spawn`.
- **Three separate batch submissions:** Don't submit clustering, summarization, and instruction extraction as separate batches. One combined prompt per conversation is 3x cheaper.
- **Blocking the main thread with polling:** The poll loop must be async and run on the background runtime. Never `.await` inside a synchronous Tauri command without `async fn`.
- **Storing the API key in SQLite or app state:** The key must only live in the macOS Keychain. Never pass it to frontend state or log it.
- **Using `token_estimate` from DB as the displayed cost:** It's a character-count approximation (char÷4). Use it only as a fast local fallback; replace with the exact count from `/v1/messages/count_tokens` before displaying the cost screen.
- **Not handling `NoEntry` from keyring:** First launch always returns an error (not a panic). Handle it as the normal "no key stored" flow.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Secure credential storage | Custom encrypted SQLite field | `keyring` crate (apple-native) | macOS Keychain handles encryption, OS-level access control, user prompts on access — all edge cases handled |
| Token counting | Character-divide-by-4 or tiktoken port | `/v1/messages/count_tokens` endpoint | Anthropic's tokenizer is not public; character estimates are off by 20-50% for multilingual content and code |
| Batch result parsing | Custom JSONL streaming | `reqwest` response body streaming + `serde_json::from_str` per line | JSONL has edge cases (embedded newlines in JSON strings) that a naive line split misses |
| Retry logic for transient errors | Custom exponential backoff | Simple 5-second poll with max-retry counter | Batch API is idempotent; 5-second poll is the user-decided interval; add a 3-retry max with user-visible "Try again" on failure |

**Key insight:** The macOS Keychain, the Batch API, and the token counting endpoint each solve their respective problems completely. No custom crypto, no tiktoken port, no hand-rolled retry state machine.

## Common Pitfalls

### Pitfall 1: Inconsistent Cluster Labels Across Batch Requests
**What goes wrong:** Each batch request is processed independently. If the clustering prompt says "choose a cluster label", different conversations may generate `"Machine Learning"`, `"ML Projects"`, `"AI/ML"` for the same conceptual group.
**Why it happens:** Batch requests have no shared state between them — they're truly independent.
**How to avoid:** Two-pass approach: Pass 1 generates the cluster vocabulary (non-batch, single request). Pass 2 embeds that vocabulary in the system prompt so all batch requests choose from the same fixed list.
**Warning signs:** Preview screen (Phase 3) showing 15+ clusters with near-identical names.

### Pitfall 2: `tokio::spawn` Panic in Tauri v2
**What goes wrong:** The poll loop crashes the app with `thread 'tokio-runtime-worker' panicked at 'no reactor running'`.
**Why it happens:** Tauri v2 manages its own tokio runtime; raw `tokio::spawn` can be called outside that runtime's context.
**How to avoid:** Always use `tauri::async_runtime::spawn` for background tasks inside Tauri commands.
**Warning signs:** App crashes silently during batch submission; no error propagated to frontend.

### Pitfall 3: API Key Validation Race
**What goes wrong:** User enters key, clicks Continue, `set_api_key` succeeds (writes to Keychain), but then the Batch API returns 401 on first use — leaving a bad key in Keychain.
**Why it happens:** The Keychain write and the API key validation are separate operations.
**How to avoid:** Validate the key BEFORE writing to Keychain. Make a single, cheap test call (e.g., a minimal count_tokens request) with the entered key. Only call `set_api_key` if that succeeds. Show "Invalid API key — check console.anthropic.com" inline if it fails.
**Warning signs:** Users report needing to manually clear Keychain entries.

### Pitfall 4: Batch Size Exceeds 256MB Limit
**What goes wrong:** A large ChatGPT export (1,000+ conversations, long full_text) might exceed the 256MB batch size limit, causing a 413 error.
**Why it happens:** The batch payload includes full conversation text in each request.
**How to avoid:** Truncate `full_text` per conversation to ~8,000 characters (roughly 2,000 tokens) before including in batch params. The summary and clustering tasks don't need the full text — just the most recent/representative portions. Also chunk into multiple batches if count > 5,000 conversations (safe margin below 100K limit).
**Warning signs:** `413 request_too_large` error from Anthropic API on batch creation.

### Pitfall 5: Results Order Not Matching Request Order
**What goes wrong:** Batch results in the JSONL file are not in the same order as the original requests. Naively indexing results by position produces wrong cluster assignments.
**Why it happens:** Batch processing is concurrent; Anthropic does not guarantee order.
**How to avoid:** Always match results to conversations via `custom_id` (which should equal `conversation_id`). Build a HashMap from `custom_id` → result before writing to SQLite.
**Warning signs:** Conversations appear with wrong summaries or cluster assignments in Phase 3 preview.

### Pitfall 6: Keyring `apple-native` Feature Flag Missing
**What goes wrong:** `keyring` crate compiles and links but silently uses the wrong backend (or panics) on macOS.
**Why it happens:** The `keyring` crate has "no default features" — macOS support requires explicitly enabling `apple-native`.
**How to avoid:** Always specify `keyring = { version = "3", features = ["apple-native"] }` in Cargo.toml.
**Warning signs:** Keychain entries not visible in macOS Keychain Access app; unexpected errors at runtime.

### Pitfall 7: Model ID for claude-haiku-3-5
**What goes wrong:** Using `"claude-haiku-3-5"` as the model ID returns a 404/invalid model error.
**Why it happens:** The full versioned model ID is required for the Batches API.
**How to avoid:** Use the exact model ID string: `"claude-haiku-3-5-20241022"`. The alias `"claude-haiku-3-5"` may work in the regular Messages API but should not be relied upon in batch requests.
**Warning signs:** Batch request `errored` results with `invalid_request` error type.

## Code Examples

### Keychain Read/Write (Rust)
```rust
// Source: https://docs.rs/keyring/3.6.3/keyring/struct.Entry.html
use keyring::Entry;

let entry = Entry::new("com.darrellwhitelaw.chatgpt-to-claude", "anthropic-api-key")?;

// Write:
entry.set_password("sk-ant-...")?;

// Read (returns Err on first launch — not a panic):
match entry.get_password() {
    Ok(key) => { /* key exists */ }
    Err(keyring::Error::NoEntry) => { /* first launch — prompt user */ }
    Err(e) => { /* real error */ }
}

// Delete:
entry.delete_credential()?;
```

### Batch Creation Request (Rust/reqwest)
```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/batch-processing
// POST https://api.anthropic.com/v1/messages/batches

let client = reqwest::Client::new();
let response = client
    .post("https://api.anthropic.com/v1/messages/batches")
    .header("x-api-key", &api_key)
    .header("anthropic-version", "2023-06-01")
    .header("content-type", "application/json")
    .json(&serde_json::json!({
        "requests": requests_array  // Vec of {custom_id, params} objects
    }))
    .send()
    .await?;

// Response when created:
// { "id": "msgbatch_xxx", "processing_status": "in_progress", ... }
```

### Batch Polling (Rust/reqwest)
```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/batch-processing
// GET https://api.anthropic.com/v1/messages/batches/{batch_id}

loop {
    let status: serde_json::Value = client
        .get(format!("https://api.anthropic.com/v1/messages/batches/{}", batch_id))
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?
        .json()
        .await?;

    if status["processing_status"] == "ended" {
        let results_url = status["results_url"].as_str().unwrap();
        // fetch JSONL results from results_url
        break;
    }

    on_event.send(ClusterEvent::Polling { elapsed_secs })?;
    tokio::time::sleep(Duration::from_secs(5)).await;  // user-decided interval
}
```

### Token Count Pre-flight (Rust/reqwest)
```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/token-counting
// POST https://api.anthropic.com/v1/messages/count_tokens
// Response: { "input_tokens": N }
// Free to use; does not consume batch quota

let response: serde_json::Value = client
    .post("https://api.anthropic.com/v1/messages/count_tokens")
    .header("x-api-key", &api_key)
    .header("anthropic-version", "2023-06-01")
    .header("content-type", "application/json")
    .json(&serde_json::json!({
        "model": "claude-haiku-3-5-20241022",
        "system": CLUSTERING_SYSTEM_PROMPT,
        "messages": [{"role": "user", "content": all_full_text_concatenated}]
    }))
    .send().await?.json().await?;

let input_tokens = response["input_tokens"].as_u64().unwrap_or(0);

// Haiku 3.5 batch pricing (source: https://platform.claude.com/docs/en/about-claude/pricing):
// input:  $0.40 / MTok  →  input_tokens * 0.40 / 1_000_000
// output: $2.00 / MTok  →  estimated_output_tokens * 2.00 / 1_000_000
// estimated_output_tokens ≈ conversation_count * 300 (summary + cluster label ~300 tokens each)
let cost = (input_tokens as f64 * 0.40 / 1_000_000.0)
         + (conversation_count as f64 * 300.0 * 2.00 / 1_000_000.0);
```

### SQLite Schema Migration
```sql
-- src-tauri/src/store/schema.sql additions
-- Existing columns cluster_id and project_name already present.
-- Add new columns for Phase 2 output:

CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at INTEGER,
    message_count INTEGER NOT NULL DEFAULT 0,
    has_images INTEGER NOT NULL DEFAULT 0,
    has_code INTEGER NOT NULL DEFAULT 0,
    token_estimate INTEGER NOT NULL DEFAULT 0,
    full_text TEXT NOT NULL DEFAULT '',
    cluster_id TEXT,           -- existing
    project_name TEXT,         -- existing
    cluster_label TEXT,        -- NEW Phase 2: human-readable cluster name
    summary TEXT,              -- NEW Phase 2: 3-5 sentence AI summary
    instructions TEXT          -- NEW Phase 2: extracted custom instructions (nullable)
);

-- Use ALTER TABLE IF NOT EXISTS pattern for columns that may already exist:
-- rusqlite does not support IF NOT EXISTS on ALTER TABLE
-- Use: CREATE TABLE with all columns, init_schema is idempotent via IF NOT EXISTS
```

### AppPhase Channel Events for Clustering
```typescript
// src/lib/bindings.ts — add to existing IngestEvent pattern
export type ClusterEvent =
  | { event: 'estimatingTokens' }
  | { event: 'tokensCounted'; data: { tokens: number; estimatedUsd: number } }
  | { event: 'batchSubmitted'; data: { batchId: string } }
  | { event: 'polling'; data: { elapsedSecs: number } }
  | { event: 'complete'; data: { assignedCount: number } }
  | { event: 'error'; data: { message: string } };
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual macOS Keychain via `security` CLI | `keyring` crate 3.x with `apple-native` | 2023+ | Pure Rust, no subprocess, synchronous API |
| Sequential Anthropic API calls for clustering | Message Batches API | Oct 2024 | 50% cost reduction, handles thousands of requests, avoids rate limits |
| `claude-3-haiku-20240307` (deprecated) | `claude-haiku-3-5-20241022` | 2024 | Haiku 3 deprecated April 19, 2026; use 3.5 now |
| `tokio::spawn` in Tauri commands | `tauri::async_runtime::spawn` | Tauri v2 | Prevents panic from missing reactor context |
| Client-side token estimation | `/v1/messages/count_tokens` (free) | 2024 | Exact count, includes system prompt tokens, free |

**Deprecated/outdated:**
- `claude-3-haiku-20240307`: Deprecated, retires April 19, 2026 — do not use even though it works today
- `claude-haiku-3-5` alias (without date): Use `claude-haiku-3-5-20241022` for stability in batch requests
- `tokio::spawn` direct: Known Tauri v2 issue; use `tauri::async_runtime::spawn`

## Open Questions

1. **Does the `keyring` crate on macOS require any macOS entitlements in the Tauri app bundle?**
   - What we know: The keyring crate wraps macOS Keychain Services directly. Entitlements for keychain-access-groups are only required for sharing keychain items across apps (app groups). For a single app accessing its own entries, no special entitlements are needed for unsigned development builds.
   - What's unclear: Whether code-signing or notarization (Phase 5) introduces additional keychain entitlement requirements.
   - Recommendation: Proceed without entitlements in Phase 2 (unsigned dev build). Flag for Phase 5 if Gatekeeper introduces keychain prompts.

2. **Should Pass 1 (cluster vocabulary discovery) be a standard Messages API call or also use the Batch API?**
   - What we know: Pass 1 needs only ONE call with a sample of all conversation titles/snippets (~100 titles, ~5K tokens). This is trivially cheap and fast as a synchronous Messages call.
   - What's unclear: Nothing — this is clearly better as a synchronous call.
   - Recommendation: Use standard POST /v1/messages for Pass 1. No batch needed.

3. **What is the `$5` warning threshold for "This is higher than typical"?**
   - What we know: Left to Claude's discretion per CONTEXT.md. Average ChatGPT export (1,000 conversations, ~2M tokens) costs approximately $0.80-$1.20 at haiku-3-5 batch pricing.
   - What's unclear: The exact percentile distribution of export sizes.
   - Recommendation: Set warning threshold at $3.00 (would represent ~7,500 conversations or ~7.5M input tokens). This is well above the typical case without being alarmist.

## Sources

### Primary (HIGH confidence)
- `https://platform.claude.com/docs/en/build-with-claude/batch-processing` — Batch API: request format, polling, results, pricing, limits (100K/256MB)
- `https://platform.claude.com/docs/en/build-with-claude/token-counting` — Token counting endpoint: format, response shape, rate limits, cost (free)
- `https://platform.claude.com/docs/en/about-claude/models/overview` — Confirmed model ID `claude-haiku-3-5-20241022`; batch pricing $0.40/$2.00 per MTok
- `https://docs.rs/keyring/3.6.3/keyring/struct.Entry.html` — Entry API: new, get_password, set_password, delete_credential; apple-native feature

### Secondary (MEDIUM confidence)
- `https://github.com/hwchen/keyring-rs` — v3.6.3 stable; cross-platform; macOS uses apple-native feature (multiple sources confirm)
- `https://github.com/HuakunShen/tauri-plugin-keyring` — Wraps same keyring crate; 14 stars, 1 contributor; last commit Dec 2024 — LOW adoption, confirmed not preferred over direct crate use
- `https://v2.tauri.app/develop/calling-rust/` — Async commands pattern with `tauri::async_runtime`

### Tertiary (LOW confidence — needs validation in implementation)
- `tauri::async_runtime::spawn` vs `tokio::spawn` panic: Multiple community reports + GitHub issue #10289; treat as confirmed for safety
- `tauri-plugin-keychain` docs.rs build failure: Reported for v2.0.2; not independently verified but consistent with low maintenance signal

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — keyring, reqwest, serde, rusqlite all verified against official docs/crates.io
- Architecture: HIGH — Batch API workflow verified end-to-end from official docs; two-pass clustering is a logical necessity based on Batch API isolation properties
- Pitfalls: HIGH for API pitfalls (verified from docs); MEDIUM for keyring entitlements (needs Phase 5 validation)

**Research date:** 2026-02-28
**Valid until:** 2026-03-28 (Batch API pricing stable; model IDs stable; keyring API stable)
