# Pitfalls Research

**Domain:** ChatGPT-to-Claude migration Mac desktop app
**Researched:** 2026-02-28
**Confidence:** MEDIUM-HIGH (critical limits verified from official docs; format internals from community sources)

---

## Critical Pitfalls

### Pitfall 1: No Official Claude.ai Projects API — You Cannot Create Projects Programmatically

**What goes wrong:**
The project plan states "official Projects API preferred; browser automation as fallback." The assumption that a Projects creation API exists is wrong. The Anthropic API currently offers: Messages, Batch, Token Counting, Models, Files (beta), and Skills (beta). There is no endpoint to programmatically create a Claude.ai Project, name it, set custom instructions, or attach files to it as knowledge base items. The Files API uploads files for use in API calls only — these files do not appear in Claude.ai's Projects knowledge base.

**Why it happens:**
Developers confuse the Files API (for API-context document references) with Claude.ai Projects (a claude.ai UI feature). The two systems are separate. The Files API stores files scoped to a workspace for inclusion in API messages via `file_id`. Claude.ai Projects is a product feature in the web interface with its own knowledge base that has no official programmatic access.

**How to avoid:**
The entire "upload to Claude.ai Projects" output strategy must be re-scoped at Phase 1 of planning. Three viable options:
1. **Browser automation** (fragile — see Pitfall 5) — automate the claude.ai web interface to create projects and upload files
2. **Files API as output** — deliver organized transcript files via the Files API, producing downloadable structured documents the user manually imports; framed as "Files ready to import"
3. **Export package** — produce a well-organized folder of text/PDF files the user drags into Claude.ai manually; provide clear step-by-step instructions

Verify at project start that no Projects API has shipped by checking `platform.claude.com/docs/en/api/overview` before writing a single line of upload code.

**Warning signs:**
- Any code file containing `claude.ai/api/projects` in a URL is using an undocumented endpoint that can break without notice
- If anyone says "just POST to the projects endpoint" without citing official Anthropic docs, it's undocumented reverse-engineered API

**Phase to address:** Phase 1 (architecture scoping) — the output delivery mechanism must be decided before any other work begins

---

### Pitfall 2: conversations.json Is a Tree, Not a List — Linear Parsers Produce Truncated or Shuffled Output

**What goes wrong:**
Developers write `conversations.forEach(c => c.messages.forEach(...))` expecting a flat message array. The actual structure uses a `mapping` object: a keyed dictionary where each node has `id`, `parent`, `children`, and `message` fields. A conversation's actual message sequence is reconstructed by starting from `current_node` and walking backward through parents, then reversing. Parsers that don't implement this traversal silently produce wrong output — missing branches, wrong order, or only showing the most recent branch of a conversation where the user tried multiple approaches.

**Why it happens:**
OpenAI's export format is optimized for their internal tree-based conversation branching (edit/regenerate creates branches). The format is not documented by OpenAI and is only understood through community reverse-engineering. The name "mapping" does not hint at "tree traversal required."

**How to avoid:**
Implement traversal explicitly:
```
function linearize(mapping, current_node):
  messages = []
  node_id = current_node
  while node_id:
    node = mapping[node_id]
    if node.message and node.message.content:
      messages.prepend(node.message)
    node_id = node.parent
  return messages
```
Write a unit test with a branched conversation fixture before building anything else on top of parsing.

**Warning signs:**
- Conversation counts look right but message counts are suspiciously low
- User messages appear but AI responses are missing from some conversations
- The final message in a conversation that had retries is not the last AI response the user saw

**Phase to address:** Phase 1 (ZIP parsing and data model) — foundational; everything else builds on this

---

### Pitfall 3: Null Fields, Empty Parts, and New Message Types Cause Silent Data Loss

**What goes wrong:**
The conversations.json format has evolved with every major ChatGPT feature launch (Canvas, Deep Research, Search, Tasks, image generation, voice messages, o1 reasoning). Parsers written against the 2023/2024 format fail silently or crash on 2025 exports. Specific null traps:
- `message.content` can be null (tool-call nodes)
- `message.content.parts` can be an empty array
- `message.content.parts[0]` can be an empty string `""`
- `message.author.role` can be `"tool"`, `"system"`, or `"memory"` — not just `"user"` and `"assistant"`
- `message.create_time` and `update_time` can be null
- DALL-E/image generation messages have a `content_type` of `"multimodal_text"` with nested image objects that have no stable connection to the exported image file on disk (the only link is file size — unreliable)

**Why it happens:**
OpenAI adds message types with each product feature without versioning the export format or documenting the schema. The format is inherently unstable.

**How to avoid:**
Treat every field access as nullable. Use defensive parsing with explicit type-checking at every level. Maintain a message type allowlist and log (don't crash) on unknown types. Instrument how many messages were skipped due to unknown type so users can see "X conversations had unsupported content types." Build a test fixture suite with exports from different time periods.

**Warning signs:**
- Parser works on your own export but fails for beta testers who have newer accounts
- Total token counts during clustering pass are suspiciously low vs. expected conversation volume
- Crashes with `TypeError: Cannot read property 'parts' of null`

**Phase to address:** Phase 1 (parsing layer) — build the defensive model before any downstream work

---

### Pitfall 4: Tier 1 API Rate Limits Will Stall Bulk Clustering of Large Exports

**What goes wrong:**
A new Anthropic account starts at Tier 1. At Tier 1, Claude Haiku 4.5 (the cheapest viable clustering model) is limited to: 50 RPM, 50,000 ITPM, 10,000 OTPM. A user with 3,000 conversations, averaging 2,000 tokens per conversation, represents 6,000,000 input tokens of clustering work. At 50,000 ITPM, that's 120 minutes of pure API time — if there are zero other constraints. In practice: burst behavior means even 50 RPM is enforced as ~1 req/second; conversation embeddings or summaries sent without batching will hit 429s immediately; the token bucket algorithm means a rapid startup burst triggers acceleration limits before settling.

**Why it happens:**
Developers test with their own small export (maybe 50 conversations), the limits don't bite, and they ship. First user with 3+ years of ChatGPT history hits a wall.

**How to avoid:**
Use the Message Batches API for all clustering/summarization work. Batch API has a separate rate limit (50 RPM for Tier 1, 100k requests per batch queue) and is priced at 50% discount. Design the clustering pipeline as: (1) pack conversations into batches of up to 100k items, (2) submit to Batch API, (3) poll for completion. Build a progress display showing "Clustering: batch 3 of 12, estimated 8 min remaining." Add user guidance: "For best results, use an API key with $40+ of prior spend (Tier 2)." For Tier 1 users, auto-select the smallest viable model (Haiku 4.5) and add inter-batch delays.

Concrete Tier 1 limits from official docs (as of 2026-02-28):
- Claude Haiku 4.5: 50 RPM, 50,000 ITPM, 10,000 OTPM
- Claude Sonnet 4.x: 50 RPM, 30,000 ITPM, 8,000 OTPM

**Warning signs:**
- 429 errors appearing within the first 60 seconds of a run
- Response headers show `anthropic-ratelimit-requests-remaining: 0`
- Clustering phase takes 10x longer than the progress bar predicted

**Phase to address:** Phase 2 (API clustering pipeline) — design for batch API from day one, never fire-and-forget sequential requests

---

### Pitfall 5: Claude.ai Browser Automation Breaks on Every UI Deploy

**What goes wrong:**
If the team decides browser automation is the upload strategy (see Pitfall 1), Playwright/Puppeteer automation against claude.ai is fragile by nature. Anthropic deploys UI changes without notice. Any automation depending on CSS selectors, element IDs, or page structure will break silently — uploads appear to succeed but files go to the wrong project, or the create-project flow changes and the app hangs. Claude.ai also has Cloudflare protection and may detect headless browser fingerprints, triggering CAPTCHAs or silent blocking.

**Why it happens:**
Browser automation is attractive because it bypasses the missing Projects API. Developers build it, it works in testing, and it ships. Then it breaks in production after the next Anthropic UI refresh.

**How to avoid:**
Do not use browser automation as a primary delivery mechanism for a tool meant to work reliably. If browser automation is implemented at all, it must be behind a feature flag, labeled "experimental," and never in the critical path of the main workflow. The primary output should be something the user controls manually (see Pitfall 1 options). If browser automation is attempted, use accessibility-tree selectors (aria labels) rather than CSS classes — more stable across visual redesigns. Build an explicit "automation health check" that runs on startup and warns the user if the detected claude.ai DOM structure doesn't match the expected version.

**Warning signs:**
- Selectors like `.ProjectCreateButton` or `[data-testid="upload-file"]` in the codebase
- No retry/detection logic for unexpected page states
- Tests only run against a recorded fixture, never live against claude.ai

**Phase to address:** Phase 3 (output delivery) — decide the delivery mechanism explicitly; don't let it default to automation

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `JSON.parse(fs.readFileSync('conversations.json'))` | Simple, fast to write | OOM crash on exports >500MB; blocks main thread | Never — always stream |
| Storing API key in `~/.chatgpt-migrator/config.json` | No dependency on Keychain | Key readable by any process; exposed in backup files | Never — use Keychain from day one |
| Fire-and-forget sequential API calls for clustering | Simpler code | Rate-limited to a crawl; no retry logic; no progress visibility | Never — use Batch API |
| Hardcode Claude Sonnet as clustering model | One less config surface | Burns 10x more tokens than Haiku 4.5 for equivalent clustering quality | Never — let user choose model |
| Skip notarization for "personal use" builds | Faster initial distribution | Gatekeeper blocks app for team members and beta testers on macOS 14+ | Only for localhost-only dev builds |
| Extract entire ZIP to temp directory before processing | Simpler to read files | Doubles disk space requirement; 10GB export = 10GB temp; fills small SSDs | Acceptable for MVP if temp is cleaned up immediately and size is shown to user |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Anthropic Messages API | Sending all conversations in a tight loop without reading rate limit response headers | Read `anthropic-ratelimit-requests-remaining` and `retry-after` headers on every response; back off before hitting 0 |
| Anthropic Batch API | Treating batch completion as synchronous; polling too frequently | Use exponential backoff polling (5s → 10s → 30s → 60s); Batch API processes asynchronously, completion can take minutes |
| Files API | Assuming files uploaded via Files API appear in Claude.ai Projects knowledge base | They do not — Files API files are scoped to API calls only, not the claude.ai UI |
| Files API | Uploading full raw conversations.json as one 500MB file | Each file is billed as input tokens per request; one 500MB plaintext = context window exceeded error |
| macOS Keychain (Electron safeStorage) | Using `node-keytar` | keytar is unmaintained since Dec 2022; use Electron's built-in `safeStorage` API instead |
| macOS Keychain (Tauri) | Expecting keychain access to be silent | Tauri keychain plugin may prompt user for permission on first access — handle the permission dialog in UX flow |
| ZIP file reading | Using `unzipper` or `adm-zip` to fully buffer a 5GB ZIP | Use streaming ZIP extraction (e.g., `yauzl` for Node.js) to avoid loading the entire archive into memory |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Loading full conversations.json with `JSON.parse()` | App hangs for 30–120 seconds; macOS beach ball; eventual OOM crash | Use `stream-json` or `JSONStream` for streaming parse | Exports with >10,000 conversations (~200MB+) |
| Sending conversation text to Claude token-by-token without batching | 50 API calls/minute at Tier 1 = 1 call/second = 50 conversations clustered per minute for a 3,000-conversation export = 60 minutes | Batch API groups thousands of calls into one async job | Any export with >100 conversations |
| Building cluster assignments in memory for all conversations simultaneously | Node.js V8 heap exhausted; renderer process OOM in Electron | Stream cluster results to disk as they arrive; never hold the full result set in memory | >5,000 conversations with summaries in memory |
| Extracting all attachments from ZIP before starting processing | Fills disk with 5–10GB of images before any work begins | Extract only what's needed, on demand, streaming | Any export with years of image attachments |
| Re-encoding conversation text as UTF-8 without normalization | Malformed filenames; broken JSON generation for special characters; emoji in conversation titles causing file write failures | Apply NFKC normalization to all text used in file names and JSON strings | Conversations with emoji titles, RTL text, or special Unicode |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Logging the API key in any debug output, crash report, or analytics event | Key exposed in log files, crash reporter uploads, or console output visible over shoulder | Audit every log statement; mask key as `sk-ant-...****` before any display |
| Storing API key in `UserDefaults`, `localStorage`, or app config file | Readable by any process with user-level access; included in Time Machine backups unencrypted | Use `safeStorage` in Electron or the Keychain plugin in Tauri exclusively |
| Passing API key as a command-line argument to subprocess | Visible in `ps aux` output to all users on the system | Pass via environment variable or stdin; never CLI args |
| Electron asar bundle without integrity verification | Attacker modifies JS files in the bundle; app still reads Keychain and exfiltrates key | Enable `asar-integrity` and Hardened Runtime; sign with `--deep` flag |
| Sending full conversation text including personal details to Claude API without user awareness | User data sent to Anthropic servers; potential ToS/privacy concerns for sensitive conversations | Make data flow explicit in onboarding; give user option to preview what gets sent; don't send more than needed for clustering (titles + short excerpts may suffice) |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Starting clustering with no progress feedback | User thinks app is frozen; force-quits; wastes API credits | Show per-conversation progress bar with estimated time; show "X of Y conversations clustered" |
| Revealing API key in the preferences UI in plaintext | User sharing screen leaks the key | Mask the key field; show `sk-ant-...****`; only reveal on explicit click |
| Showing "Upload complete" when only Files API upload succeeded (not Projects) | User opens Claude.ai, finds no new projects, thinks the app lied | Be explicit about what the output is: "Files prepared for import" not "Uploaded to Claude.ai" |
| Starting a 3,000-conversation run with no way to cancel | User must force-quit; partial state leaves uploaded files orphaned in Files API workspace | Implement cancel button that stops the batch, cleans up temp files, and shows what completed |
| Not showing estimated cost before clustering begins | User gets a $20 surprise bill on their first run | Calculate approximate token count from conversations.json before calling any API; show estimated cost and require confirmation |

---

## "Looks Done But Isn't" Checklist

- [ ] **ZIP parsing:** Handles Safari's auto-unzip behavior (export arrives as folder, not .zip) — verify both paths
- [ ] **conversation.json parsing:** Handles null `create_time`, null `content`, empty `parts` arrays — verify with edge case fixtures
- [ ] **Branching conversations:** The message selected is the one the user saw last (via `current_node`), not the first generated response — verify with a branched conversation fixture
- [ ] **Rate limiting:** 429 responses trigger retry with `retry-after` delay, not immediate re-throw — verify by mocking a 429
- [ ] **Keychain storage:** API key survives app restart and is read from Keychain, not re-asked on every launch — verify by quitting and reopening
- [ ] **Code signing:** App opens on a clean macOS install without "damaged" or "unverified developer" Gatekeeper warnings — test on a machine that has never run the app in dev mode
- [ ] **Notarization stapling:** Notarization ticket is stapled to the DMG, so it works offline without Apple server check — verify with `spctl --assess -v` offline
- [ ] **Temp file cleanup:** Large ZIP extractions in `/tmp` are deleted after processing, even if the run fails midway — verify by interrupting a run and checking disk
- [ ] **Cost estimate:** Shown before any API call is made, not after the first batch starts — verify by watching network tab during onboarding

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| No Projects API — wrong output strategy | HIGH | Redesign output as local file export + manual import instructions; discard any automation code |
| Linear parser producing wrong output | MEDIUM | Rewrite traversal using `current_node` + parent chain; re-test against all user export fixtures |
| Tier 1 rate limits stalling production users | MEDIUM | Add Batch API support as priority hotfix; add tier-level detection and model auto-selection |
| Browser automation broken by Anthropic UI update | MEDIUM | Ship emergency update switching to file export mode; remove automation path entirely |
| API key leaked in log file | HIGH | Immediately instruct user to revoke key in Anthropic Console; audit all log outputs; release patch |
| OOM crash on large export | LOW | Replace `JSON.parse` with streaming parser; ship patch within hours of first report |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| No Projects API | Phase 1 (Architecture) | Decision doc confirms output strategy with no undocumented endpoints |
| Tree traversal for conversations.json | Phase 1 (ZIP + Parsing) | Unit tests pass with branched conversation fixture |
| Null fields and new message types | Phase 1 (ZIP + Parsing) | Defensive parser handles null at every level; unknown types logged not crashed |
| Tier 1 rate limits at scale | Phase 2 (Clustering pipeline) | Batch API used; Tier 1 user with 3,000-conversation export completes without 429 crash |
| Browser automation fragility | Phase 3 (Output delivery) | Output strategy decision explicitly avoids automation as primary path |
| Memory exhaustion on large ZIPs | Phase 1 (ZIP + Parsing) | Streaming parser confirmed; 1GB ZIP processed with <100MB heap |
| Mac distribution / Gatekeeper | Phase 4 (Distribution) | Fresh-machine install test passes; notarization verified offline |
| API key Keychain security | Phase 2 (API integration) | safeStorage used; no key in logs, config files, or environment at rest |
| Estimated cost transparency | Phase 2 (Clustering pipeline) | Cost estimate shown before first API call; confirmed by UX review |

---

## Sources

- Anthropic Rate Limits (official, verified 2026-02-28): https://platform.claude.com/docs/en/api/rate-limits
- Anthropic Files API (official, verified 2026-02-28): https://platform.claude.com/docs/en/build-with-claude/files
- Anthropic API Overview — no Projects API listed (official, verified 2026-02-28): https://platform.claude.com/docs/en/api/overview
- OpenAI Community: conversations.json structure & branching tree: https://community.openai.com/t/decoding-exported-data-by-parsing-conversations-json-and-or-chat-html/403144
- OpenAI Community: JSON structure questions (null fields, format changes): https://community.openai.com/t/questions-about-the-json-structures-in-the-exported-conversations-json/954762
- Electron safeStorage (official): https://www.electronjs.org/docs/latest/api/safe-storage
- Replacing keytar with safeStorage (real-world migration): https://freek.dev/2103-replacing-keytar-with-electrons-safestorage-in-ray
- Keychain security vulnerability in Electron apps (security research): https://wojciechregula.blog/post/stealing-macos-apps-keychain-entries/
- Electron OOM memory limits (GitHub issues): https://github.com/electron/electron/issues/31330
- Tauri macOS code signing guide (official): https://v2.tauri.app/distribute/sign/macos/
- macOS notarization common issues (Apple developer docs): https://developer.apple.com/documentation/security/resolving-common-notarization-issues
- Electron macOS code signing guide (2025): https://securityboulevard.com/2025/12/how-to-code-signing-an-electron-js-app-for-macos/

---
*Pitfalls research for: ChatGPT → Claude migration Mac desktop app*
*Researched: 2026-02-28*
