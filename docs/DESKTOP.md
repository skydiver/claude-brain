# Desktop App

A native macOS desktop application for browsing claude-brain knowledge entries. Built with Tauri v2 and Leptos 0.8.

## Overview

The MCP server gives Claude a persistent knowledge store, but there's no way for humans to browse that knowledge outside of asking Claude. This app provides a native desktop interface for searching, filtering, and reading entries — like a notebook backed by the brain's SQLite database.

### Features

- **Three-pane layout:** sidebar filters, entry list, detail view
- **Full-text search** via FTS5 with 300ms debounced input
- **Filter by** type (learning, gotcha, project context), technology, and tags
- **Rendered markdown** with tables, code blocks, and strikethrough
- **System theme** — follows macOS light/dark mode
- **Keyboard shortcuts:** `Cmd+K` (search), `Escape` (clear), `↑`/`↓` (navigate)
- **Auto-refresh** when the window regains focus

### What This Is NOT

- Not a replacement for the MCP server — Claude still writes entries via MCP
- Not an editor (read-only MVP)
- Not a sync tool or multi-device solution

## Build & Run

### Development

```bash
make dev
```

This starts trunk (WASM hot-reload on `:1420`) and opens a native Tauri window.

### Production build

```bash
make brain-app
```

Output: `dist/ClaudeBrain.app`

Open it with:

```bash
open dist/ClaudeBrain.app
```

## Architecture

Two crates in the workspace:

| Crate                | Role                                                                                        |
| -------------------- | ------------------------------------------------------------------------------------------- |
| `brain-app`          | Tauri v2 native shell — opens the database via `brain-core`, exposes read-only IPC commands |
| `brain-app-frontend` | Leptos 0.8 WASM UI — three-pane layout with Tailwind CSS                                    |

```
Tauri (native shell) ──manages──▶ Webview
  │                                  │
  │ Rust: brain-core → SQLite        │ WASM: Leptos + Tailwind
  │                                  │
  └──── Tauri IPC (commands) ────────┘
```

Both the MCP server and the desktop app can open the same SQLite database concurrently — WAL mode (enabled by brain-core) supports concurrent readers.

### IPC Commands

All commands are read-only:

| Command               | Description                       |
| --------------------- | --------------------------------- |
| `list_entries`        | Paginated list with filters       |
| `search_entries`      | FTS5 full-text search             |
| `get_entry`           | Single entry by ID                |
| `get_project_context` | Entries scoped to a project path  |
| `list_technologies`   | Distinct technologies             |
| `list_tags`           | Distinct tags                     |
| `stats`               | Counts by type and recent entries |

### Database

Same database as the MCP server: `~/.config/claude-brain/brain.db`

Override with `BRAIN_DB_PATH` environment variable. If the database doesn't exist, the app creates it with the full schema.

## Prerequisites

These are installed globally (one-time):

```bash
cargo install tauri-cli          # cargo tauri subcommand
cargo install trunk              # WASM bundler for Leptos
rustup target add wasm32-unknown-unknown  # WASM compilation target
```

Node dependencies (in `crates/brain-app-frontend/`):

```bash
cd crates/brain-app-frontend
npm install
```
