# Stack Research

**Domain:** Mac desktop data migration tool (ChatGPT exports → Claude.ai Projects)
**Researched:** 2026-02-28
**Confidence:** MEDIUM-HIGH (Claude.ai Projects API gap is the main unresolved uncertainty)

---

## Critical Finding: Claude.ai Projects Has No Public API

**Before any stack discussion, this must be stated clearly.**

As of February 2026, there is no official REST API for creating or managing Claude.ai Projects in the web interface. The Anthropic Files API (`/v1/files`, still in beta as of this research) is an API for uploading files to be used *within the Claude messaging API* — it does NOT create Projects in the claude.ai web UI. Multiple sources confirm programmatic project creation is not exposed.

**Implication for the app:** The "upload to Claude.ai Projects" feature requires one of:

1. **Unofficial browser automation** (Playwright/Puppeteer automating claude.ai) — fragile, may break on any UI change, against ToS risk
2. **Export-to-import file format** — generate a structured export that users manually import
3. **Reframe the output target** — use the Anthropic Messages API itself (user provides API key), generate well-structured markdown conversation files the user can manually upload
4. **Wait/poll for official API** — Anthropic does not have a public roadmap for this

The roadmap must treat the "upload to Claude.ai Projects" requirement as a **research spike** before implementation, not a done deal. This is the highest-risk item in the project.

---

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Tauri | 2.4.x (latest stable) | Desktop app shell, OS integration | Smallest binary (~5 MB vs Electron's ~150 MB), uses macOS native WebKit for UI rendering (genuine native feel), Rust backend handles file I/O without memory bloat, direct access to macOS Keychain via community plugin |
| React | 19.x | UI framework | Default template for Tauri; massive ecosystem; TypeScript-first; the team already knows it given project context |
| TypeScript | 5.x | Type safety across frontend and Tauri invoke bridge | Tauri 2.0 generates typed bindings via `tauri-specta`, making the Rust↔TS bridge type-safe |
| Vite | 6.x | Frontend bundler | Official Tauri bundler integration; HMR for dev; fastest cold starts |

### AI / API Layer

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| `@anthropic-ai/sdk` | 0.78.x | Claude API calls (clustering, summarization) | Official Anthropic SDK, actively maintained (released 2026-02-19), full TypeScript types, streaming support via SSE, message batches for high-volume clustering |
| `@anthropic-ai/sdk` (Files API beta) | 0.78.x | Upload conversation transcripts as files | The Files API (`anthropic-beta: files-api-2025-04-14`) supports up to 500 MB per file and 100 GB org storage — this is the best available official API for durable file storage |

**Note:** The Files API creates files scoped to your API workspace, not to claude.ai Projects. Files uploaded here are accessible in the Claude API Messages context, not visible in the claude.ai web UI. The app must be honest about this distinction with users.

### ZIP Processing

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| `unzipper` | 0.12.x | Stream-extract large ZIP archives | The only actively maintained, ZIP64-compliant, streaming-capable Node.js unzip library. Uses pull-streams (`Open`) and push-streams (`Extract`) — never reads entire ZIP into memory. Critical for ChatGPT exports that can be many GB. |

**Rejected alternatives:**
- `node-stream-zip` — last published 4 years ago (v1.15.0), maintenance status unclear
- `adm-zip` — synchronous, loads entire file into memory — instant disqualification for large files
- `yauzl` — well-designed but lower-level, requires manual security (path traversal) implementation

### UI Components

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| `shadcn/ui` | latest | Component primitives | Runs on Radix UI primitives; copy-into-project model means no version lock-in; works perfectly with Tauri + Vite; community has Tauri-specific templates |
| `tailwindcss` | 4.x | Styling | Tight integration with shadcn/ui; v4 has improved performance; the natural companion to shadcn |
| `lucide-react` | 0.4xx | Icons | Ships with shadcn/ui templates; consistent icon system |

### Credential Storage

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| `tauri-plugin-keyring` | community (latest) | Secure API key storage in macOS Keychain | The PROJECT.md requirement: "Claude API key entered by user at runtime, stored in macOS Keychain — never in code or files." The Tauri Stronghold plugin is deprecated (will be removed in v3). The `tauri-plugin-keyring` wraps the Rust `keyring` crate, which calls macOS Security framework directly. |

**Risk:** This is a community plugin, not official Tauri. Verify activity before adopting. Alternative: use the Tauri Rust backend directly with the `keyring` crate (no plugin needed — write a small Tauri command).

### Progress / State

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| `zustand` | 4.x | App-wide state (import progress, phase tracking) | Lightweight (~1 KB), no boilerplate, works well with React 19; perfect for single-session state (no persistence needed — the app is one-shot per the PROJECT.md out-of-scope list) |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `pnpm` | Package manager | Faster than npm, works with Tauri; standard for monorepo-friendly setups |
| `vitest` | Unit testing | Same config as Vite; test ZIP parsing logic and clustering prompts in isolation |
| `biome` | Linting + formatting | Single tool replacing ESLint + Prettier; much faster; growing adoption in 2026 |
| `tauri-specta` | Type-safe Rust↔TS bridge | Generates TypeScript types from Tauri command signatures — eliminates a whole class of runtime errors at the IPC boundary |

---

## Alternatives Considered: Tauri vs Electron vs SwiftUI

### Quick Comparison

| Criterion | Tauri 2.x | Electron 33.x | SwiftUI (native) |
|-----------|-----------|---------------|-----------------|
| Bundle size | ~5–15 MB | ~150–200 MB | ~10–30 MB |
| Memory at startup | 20–40 MB | 200–400 MB | 15–30 MB |
| Startup time | <0.5 s | 1–2 s | <0.3 s |
| macOS native feel | HIGH (uses WebKit) | MEDIUM (Chromium) | HIGHEST (AppKit/SwiftUI) |
| TypeScript support | NATIVE (frontend) | NATIVE | None |
| Team velocity | HIGH (web skills transfer) | HIGH | LOW (requires Swift expertise) |
| Rust required | YES (Tauri backend) | NO | NO |
| Keychain access | Community plugin | Node.js `keytar` | Native API |
| ZIP streaming | `unzipper` via Node.js | `unzipper` via Node.js | Custom Swift code |
| Anthropic SDK | `@anthropic-ai/sdk` | `@anthropic-ai/sdk` | HTTP calls only |
| Playwright automation | Available | Available | Via scripting bridge |
| Distribution without App Store | DMG + notarization | DMG + notarization | DMG + notarization |
| Open source friendliness | HIGH | MEDIUM (large binary) | MEDIUM |

### Recommendation: Tauri

Use **Tauri** because:

1. **Performance matches the use case.** ZIP files can be gigabytes. Loading a 2 GB archive in Electron would cause memory pressure because the main process and renderer share a Chromium engine. Tauri delegates file I/O to Rust, which handles streaming natively and never fights with a browser engine for memory.

2. **WebKit = genuinely macOS-native feel.** Tauri renders using WebKit (the same engine as Safari), not a bundled Chromium. macOS users see system fonts, system scrollbar behavior, and standard window chrome. Electron apps always have a slightly "off" feel that power users notice.

3. **TypeScript frontend is a natural fit.** The app's core complexity is in the JavaScript/TypeScript layer: parsing JSON, calling the Anthropic SDK, orchestrating progress updates. Rust handles only OS-level work (file I/O, keychain). You get the best of both languages in their natural domain.

4. **Distributable without App Store.** Tauri produces a signed DMG with Apple notarization via `tauri build`. Small team distribution (share a link to the DMG) works without any App Store overhead.

5. **Open source ready.** A 10 MB DMG is trivially redistributable. An Electron app at 200 MB has meaningful download friction for open source adoption.

**Why not Electron:**
- Memory overhead is a real problem for GB-scale ZIP processing
- Chromium-based rendering is not "native-feeling" on macOS
- 10x larger binary with no benefit for this use case

**Why not SwiftUI:**
- Requires Swift expertise the project doesn't have
- No `@anthropic-ai/sdk` — HTTP calls only
- Much slower iteration on UI (no HMR, no browser devtools)
- No clear advantage over Tauri for this specific tool

**Choose SwiftUI when:** Building something deeply integrated with macOS system APIs (iCloud Drive sync, Share Sheet, Shortcuts app, widgets, menu bar items tied to system state). This app needs none of those.

**Choose Electron when:** You need absolute cross-platform consistency of rendering (financial dashboards, pixel-perfect design tools) or you have existing Node.js backend code that must run in-process with no Rust boundary.

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `adm-zip` | Synchronous; loads entire ZIP into memory — will OOM on a 2 GB export | `unzipper` |
| `node-stream-zip` | Last published 4 years ago; maintenance status unclear | `unzipper` |
| Tauri `stronghold` plugin | Deprecated, will be removed in Tauri v3 | `tauri-plugin-keyring` or direct `keyring` Rust crate |
| Electron | 200 MB binary, Chromium memory overhead defeats streaming efficiency | Tauri |
| Hardcoded API key | Security requirement from PROJECT.md — never do this | macOS Keychain via `tauri-plugin-keyring` |
| Playwright to automate claude.ai | Fragile against UI changes, potential ToS violation, breaks on login changes | Wait for official API; export structured files instead |
| `@anthropic-ai/sdk` Files API as "Projects API" | The Files API does NOT create Claude.ai Projects — it uploads to the messaging API workspace only | Understand the distinction; design output format accordingly |
| Redux / MobX | Heavy state management overkill for a one-shot migration tool | `zustand` |

---

## Stack Patterns by Variant

**If Claude.ai Projects API becomes available (official):**
- Add an HTTP client layer that calls the Projects API endpoints
- Keep the ZIP parsing and clustering pipeline unchanged
- The Anthropic SDK may add first-party Projects support — monitor `@anthropic-ai/sdk` releases

**If the app pivots to "generate importable files" instead of direct upload:**
- ZIP output containing structured markdown files per conversation
- Users drag the output ZIP into claude.ai manually
- Removes the entire automation risk while preserving value

**If open sourced:**
- Tauri's MIT license and small binary size make GitHub release distribution trivial
- Consider `tauri-action` GitHub Action for CI/CD builds across architectures

**If multi-architecture macOS builds are needed (Intel + Apple Silicon):**
- Tauri supports `--target universal-apple-darwin` for universal binaries
- Known issues exist with Tauri v2 universal binaries (tracked in tauri-apps/tauri#9748) — test this early, build separate arch-specific DMGs as fallback

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `@tauri-apps/api@^2` | Tauri CLI `2.4.x` | Major versions must match |
| `react@^19` | `vite@^6` | React 19 requires Vite 5.4+ |
| `tailwindcss@^4` | `shadcn/ui` (latest) | shadcn now ships Tailwind v4-compatible components |
| `@anthropic-ai/sdk@^0.78` | Node.js `>=20 LTS` | SDK drops older Node versions; Tauri's Node.js sidecar must be 20+ |
| `unzipper@^0.12` | Node.js `>=18` | Works with Tauri's Node.js environment |

---

## Installation

```bash
# Scaffold Tauri + React + TypeScript
pnpm create tauri-app chatgpt-to-claude --template react-ts

# Or use production-ready template (recommended)
# https://github.com/dannysmith/tauri-template (Tauri v2 + React 19 + TypeScript + tauri-specta)

# Anthropic SDK
pnpm add @anthropic-ai/sdk

# ZIP streaming
pnpm add unzipper
pnpm add -D @types/unzipper

# UI
pnpm dlx shadcn@latest init
pnpm add lucide-react

# State
pnpm add zustand

# Credential storage (community plugin)
# Follow: https://github.com/HuakunShen/tauri-plugin-keyring
pnpm add tauri-plugin-keyring-api

# Dev dependencies
pnpm add -D vitest @vitest/ui tailwindcss biome tauri-specta
```

---

## Sources

- Tauri v2 official docs: https://v2.tauri.app/ — version 2.4.2 confirmed current
- Anthropic SDK TypeScript releases: https://github.com/anthropics/anthropic-sdk-typescript/releases — v0.78.0 released 2026-02-19
- Anthropic Files API docs: https://platform.claude.com/docs/en/build-with-claude/files — beta header required, NOT a Projects API
- DoltHub Electron vs Tauri comparison (Nov 2025): https://www.dolthub.com/blog/2025-11-13-electron-vs-tauri/ — MEDIUM confidence
- gethopp.app Tauri vs Electron performance: https://www.gethopp.app/blog/tauri-vs-electron — startup times, memory figures — MEDIUM confidence
- unzipper npm: https://www.npmjs.com/package/unzipper — actively maintained, ZIP64 compliant
- node-stream-zip npm: last published 4 years ago (v1.15.0) — MEDIUM confidence (maintenance concern)
- tauri-plugin-keyring: https://github.com/HuakunShen/tauri-plugin-keyring — community plugin, verify activity
- Tauri universal binary discussion: https://github.com/orgs/tauri-apps/discussions/9419 — known issues, verify before shipping
- Claude.ai Projects API absence: confirmed via multiple searches, no official endpoint exists as of 2026-02-28 — HIGH confidence
- claude-pyrojects (unofficial): https://github.com/hcevikdotpy/claude-pyrojects — existence confirms gap, affirms unofficial approach is fragile
- Tauri macOS code signing guide (2025): https://dev.to/0xmassi/shipping-a-production-macos-app-with-tauri-20-code-signing-notarization-and-homebrew-mc3

---

*Stack research for: ChatGPT to Claude Mac Desktop Migrator*
*Researched: 2026-02-28*
