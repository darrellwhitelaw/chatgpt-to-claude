# Move to Claude

A free macOS app that converts your ChatGPT export into a searchable archive Claude Desktop can read, navigate, and surface - entirely on your machine.

**[movetoclaude.com](https://movetoclaude.com)**

---

## What it does

Drop your ChatGPT Export .zip into the app. The app converts your history to clean markdown, writes a `START_HERE.md` that guides Claude through your archive, and configures a local MCP server so Claude Desktop can read everything directly on your machine.

From there, Claude reads your full history, maps every conversation to a project, and walks you through sorting your files into labeled project folders - one folder per project, each one a drag away from becoming a Claude Project.

## Requirements

- macOS, Apple Silicon
- [Claude Desktop](https://claude.ai/download)
- ChatGPT data export (from Settings on chatgpt.com)

## Install

Download the latest DMG from [Releases](https://github.com/darrellwhitelaw/chatgpt-to-claude/releases/latest), open it, and run `Install.command`.

## Privacy

Everything happens on your machine. Your export ZIP never leaves your computer. No account, no upload, no third party.

## Stack

- Tauri v2 (Rust backend)
- React + TypeScript + Vite (frontend)
- Zustand (state management)
- Tailwind CSS

## License

MIT
