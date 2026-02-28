# Phase 1: ZIP Parsing Foundation - Context

**Gathered:** 2026-02-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the Tauri Mac app scaffold + a streaming ZIP/JSON parser that turns a ChatGPT export into a SQLite conversation store. Deliver a drag-and-drop import UI with progress feedback and a completion summary. Clustering, AI processing, and output generation are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Visual Theme & Identity
- Always light — white/minimal, not dark
- Reference: context-pack.com aesthetic (spacious, clean, minimal copy) but light not dark
- Purple accent color is NOT used — stay minimal white/neutral
- Feels like a focused utility tool, not a workspace

### Window Behavior
- Fixed compact window, approximately 600×500px
- Not resizable — utility app feel
- Centers on launch

### Drop Zone Design
- Dedicated centered drop zone with dashed border and icon
- NOT full-window drop target
- Minimal copy: one line of instruction ("Drop your ChatGPT export here")
- Nothing else shown on the empty state — no step indicators, no marketing

### File Picker
- "Browse" / "select file" link sits inside the drop zone as secondary affordance
- Drop zone is primary; file picker is the "or" fallback
- No separate button below the zone

### Progress States
- Simple stage labels + spinner — no numbers, no byte counts
- Human-readable stages: e.g. "Extracting ZIP…", "Parsing conversations…", "Building index…"
- Non-technical — zero jargon

### Completion State
- Summary card: "Found [N] conversations ([year range])"
- Single prominent Continue button
- No list of conversation titles — just the aggregate summary

### Error Handling
- Inline error message — shown within or adjacent to the drop zone
- Drop zone resets immediately so user can try again without restarting
- No dedicated error screen

### Claude's Discretion
- Exact dashed border styling (dash pattern, corner radius)
- Drop zone icon/illustration
- Exact padding, typography scale
- Stage label wording (can refine as long as it's non-technical and human-readable)
- Transition animations between states

</decisions>

<specifics>
## Specific Ideas

- Primary reference: context-pack.com — clean 3-step pipeline feel, spacious layout, minimal copy
- "I want this process but in a simple UI for less technical users" — the whole thing should feel approachable, not developer tooling
- Light theme specifically (user explicitly chose this over dark and native)
- Non-technical users are the audience for this phase's UI — no byte counts, no file paths, no technical jargon in any copy

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-zip-parsing-foundation*
*Context gathered: 2026-02-28*
