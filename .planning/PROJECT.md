# ChatGPT → Claude Migrator

## What This Is

A Mac desktop app that takes your OpenAI data export ZIP (however large), uses AI to intelligently cluster your conversations by topic, and creates organized Claude.ai Projects with full transcripts, summaries, extracted code, and custom instructions — making migration feel seamless rather than lossy.

Built first for personal use, then team distribution, then potential open source release.

## Core Value

Drop in your ChatGPT export and end up with a well-organized Claude.ai Project structure that feels like your history was always there — not dumped in bulk.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can drag and drop (or select) a ChatGPT export ZIP of any size
- [ ] App extracts and parses conversations.json from the ZIP without loading entire file into memory
- [ ] App uses Claude API to cluster conversations by topic into logical project groups
- [ ] App creates a preview showing proposed project names and conversation counts before any upload
- [ ] User can trigger one-click upload to create Claude.ai Projects from the preview
- [ ] Each Claude Project receives: full conversation transcripts, AI-generated summaries, extracted code snippets, and custom instructions/system prompts
- [ ] Images and file attachments are included alongside transcripts where Claude.ai supports them
- [ ] App handles large exports gracefully (streaming, chunking, progress feedback)
- [ ] UI is simple, clean, and native-feeling on macOS

### Out of Scope

- Migration history / undo log — one-shot tool, no persistent state needed
- Local searchable library — destination is Claude.ai Projects only
- Windows / Linux support — Mac-first, cross-platform later if open sourced
- Real-time sync or incremental imports — full migration per run

## Context

- OpenAI export format: ZIP containing `conversations.json` (array of conversations with messages, titles, timestamps), `chat.html`, and media attachments
- ChatGPT exports can be very large — years of history with embedded images; must handle streaming/chunking
- Claude.ai Projects allow uploading files as knowledge docs; exact API availability TBD (research phase to determine best approach: official API vs browser automation)
- Primary user is the project owner; secondary users are teammates; tertiary is open source community
- App needs a Claude API key (Anthropic) for clustering and summarization — must be entered by user, never hardcoded

## Constraints

- **Platform**: macOS — native feel required; framework TBD by research (Tauri, Electron, or native Swift)
- **Security**: Claude API key entered by user at runtime, stored in macOS Keychain — never in code or files
- **Scale**: Must handle exports with thousands of conversations and gigabytes of attachments
- **Claude.ai integration**: Method TBD by research — official Projects API preferred; browser automation as fallback

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| AI-cluster by topic (not mirror ChatGPT structure) | User's ChatGPT org is messy; want a fresh meaningful structure | — Pending |
| Research Claude.ai Projects API before choosing integration approach | Don't want to build on fragile browser automation if official API exists | — Pending |
| One-shot importer (no persistent state) | Keeps the app simple and focused; can always add history later | — Pending |
| Include images where Claude.ai supports them | Lossy migration is worse than slightly complex migration | — Pending |

---
*Last updated: 2026-02-28 after initialization*
