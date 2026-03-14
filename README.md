# claude-brain

A persistent knowledge store for Claude Code. Store learnings, gotchas, and project context in a SQLite database — then browse them from a native desktop app or let Claude access them automatically via MCP.

## Components

| Component       | Description                                                                       | Docs                               |
| --------------- | --------------------------------------------------------------------------------- | ---------------------------------- |
| **MCP Server**  | Claude Code integration — stores and retrieves entries via Model Context Protocol | [docs/MCP.md](docs/MCP.md)         |
| **Desktop App** | Native macOS app for browsing, searching, and reading entries                     | [docs/DESKTOP.md](docs/DESKTOP.md) |

## Quick Start

```bash
# Build everything into dist/
make

# Or build individually
make brain-mcp    # MCP server binary
make brain-app    # Desktop app (ClaudeBrain.app)
```

## Development

### Prerequisites

```bash
# Rust tooling
cargo install tauri-cli
cargo install trunk
rustup target add wasm32-unknown-unknown

# Node dependencies (for Tailwind CSS)
cd crates/brain-app-frontend && npm install
```

### Commands

| Command      | Description                      |
| ------------ | -------------------------------- |
| `make`       | Build both binaries into `dist/` |
| `make dev`   | Run desktop app with hot-reload  |
| `make test`  | Run all workspace tests          |
| `make clean` | Remove all build artifacts       |

### Project Structure

```
claude-brain/
├── crates/
│   ├── brain-core/           # Shared library: SQLite, models, FTS5
│   ├── brain-mcp/            # MCP server binary (stdio transport)
│   ├── brain-app/            # Tauri v2 native shell
│   └── brain-app-frontend/   # Leptos 0.8 WASM frontend
├── docs/
│   ├── MCP.md                # MCP server documentation
│   └── DESKTOP.md            # Desktop app documentation
├── Makefile                  # Build targets
└── dist/                     # Build output (gitignored)
```

### Database

All components share the same SQLite database at `~/.config/claude-brain/brain.db` (override with `BRAIN_DB_PATH`). WAL mode enables concurrent access — the MCP server can write while the desktop app reads.

### Testing

```bash
make test
```

Runs 45 tests across the workspace: brain-core (30), brain-app (7), brain-app-frontend (8).
