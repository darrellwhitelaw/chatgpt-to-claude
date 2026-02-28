# Phase 2: API Key + AI Clustering - Context

**Gathered:** 2026-02-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Securely capture and store the Anthropic API key (macOS Keychain), run AI-powered conversation clustering via the Message Batches API, generate per-conversation summaries, and extract custom instructions. Phase ends when clustered + summarized data is stored and ready for the Phase 3 preview screen. Output folder generation is Phase 4.

</domain>

<decisions>
## Implementation Decisions

### API Key Entry Screen
- Appears **after the summary card** — user sees "Found N conversations", clicks Continue, then hits key entry if no key is stored
- Minimal UI: password-style input + label + Continue button only. No upfront cost warning on this screen
- User can **change the key** from the summary card screen (small "change key" link)
- Invalid/rejected key shows an **inline error under the input field**: "Invalid API key — check console.anthropic.com". Stay on same screen, no modal

### Cost Estimate Screen
- Shows: **token count + dollar estimate** — e.g. "~2.4M tokens · estimated $1.20"
- Two actions: **Proceed / Cancel**. Cancel returns to the summary card
- Model: **claude-haiku-3-5 for everything** (clustering + summaries). Cheapest, fast batch processing, sufficient quality
- If estimated cost exceeds a threshold (e.g. $5): show a **warning callout** — "This is higher than typical — your export is large." Let user decide without alarm

### Clustering Progress UI
- **Spinner + stage label** — same pattern as Phase 1 ZIP parsing. e.g. "Clustering conversations...", "Generating summaries..."
- User must **stay in the app** — no background/resume support in v1. Clustering should complete within ~5 minutes for typical exports
- On batch failure: **error screen with "Try again"** — returns user to cost estimate screen to resubmit
- Polling: **every 5 seconds** until the batch completes

### Conversation Summaries
- Each summary contains: **key decisions + conclusions + main topic**
- Length: **3–5 sentences** — skimmable but contextually useful
- Custom instructions extraction: **system prompts + explicit user instruction patterns** (e.g. "always respond in bullet points") found in the conversation, not just role=system messages
- Cluster count: **dynamic** — let Claude decide 5–20 clusters based on actual content distribution. No fixed target, no user input required

### Claude's Discretion
- Exact Keychain integration library/approach (tauri-plugin-keychain or native)
- Prompt design for clustering and summarization
- Exact threshold for "high cost" warning
- Token estimation approach (pre-flight vs exact count from API response)

</decisions>

<specifics>
## Specific Ideas

- The app's visual style is consistent throughout: always light theme, Inter font, neutral palette. Key entry and cost screens should match the drop zone's clean minimal aesthetic
- Stage labels during progress should be plain English, no jargon — "Clustering conversations...", "Generating summaries...", "Extracting instructions..."
- The "change key" affordance on the summary card should be subtle — not a prominent button, more like a secondary link

</specifics>

<deferred>
## Deferred Ideas

- Background batch processing with resume on reopen — Phase 2+ or v2
- User-configurable cluster target count — noted in REQUIREMENTS.md as CLUST-01 (v2)
- Model selection UI — keep single model for v1, may revisit in v2
- Processing a subset of conversations (cap by count/date) — v2 scope

</deferred>

---

*Phase: 02-api-key-ai-clustering*
*Context gathered: 2026-02-28*
