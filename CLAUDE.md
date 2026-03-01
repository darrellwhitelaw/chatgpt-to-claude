# ChatGPT to Claude

## Project Type
This is a **Tauri native macOS app** — not a web app. Browser-based preview tools (`preview_start`, `preview_screenshot`, etc.) do not apply and should not be used.

## Verification Workflow
After editing code, verify by building and running the native app:
```bash
npm run tauri build
open src-tauri/target/release/bundle/macos/ChatGPT\ to\ Claude.app
```

For quick UI iteration, the Vite dev server can be started separately:
```bash
npm run dev
```
But UI components that depend on Tauri APIs (file drop, invoke calls) will not work in a browser — only use the dev server for pure layout/style verification.

## Key Commands
- **Build app + DMG:** `npm run tauri build`
- **Build custom DMG:** `bash build-dmg.sh`
- **Push release:** `gh release create v0.1.0 ... --repo darrellwhitelaw/chatgpt-to-claude`

## Stack
- Tauri v2 (Rust backend)
- React + TypeScript + Vite (frontend)
- SQLite via rusqlite (local conversation storage)
- Zustand (state management)
- Tailwind CSS
