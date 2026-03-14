# MCP Server

A persistent knowledge store for Claude Code, exposed as an [MCP server](https://modelcontextprotocol.io/). Claude remembers what it learns across sessions — gotchas, project context, technical knowledge — and retrieves it automatically via full-text search.

## Setup

### 1. Build

```bash
make brain-mcp
```

Or directly:

```bash
cargo build --release -p brain-mcp
```

### 2. Register with Claude Code

```bash
claude mcp add --scope user brain -- /absolute/path/to/target/release/claude-brain
```

Replace `/absolute/path/to` with the actual path to this repo.

### 3. Allow tools without prompting (optional)

By default, Claude Code asks permission each time a brain tool is used. To allow all brain tools globally, add `"mcp__brain__*"` to the `permissions.allow` array in `~/.claude/settings.json`:

```json
"permissions": {
  "allow": [
    "mcp__brain__*"
  ]
}
```

### 4. Restart Claude Code

Close and reopen Claude Code (or start a new session). The brain MCP server starts automatically. You can verify it's connected — Claude will have access to the `brain_*` tools.

### 5. Start using it

The tools are available immediately. You can ask Claude to store and recall knowledge naturally:

- _"Remember that WKWebView needs the network.client entitlement even for local HTML on macOS sandbox"_
- _"What do you know about SQLite gotchas?"_
- _"What's the project context for this repo?"_
- _"Store this as a learning: Rust's orphan rule means..."_

## How It Works

claude-brain is an MCP server that runs as a subprocess of Claude Code. It communicates over stdio using the Model Context Protocol, giving Claude access to a SQLite database for persistent memory.

### What Gets Stored

Every entry has a **type**, **title**, **content** (markdown), and optional metadata (technology, project, tags).

| Type              | When to use                                   | Example                                               |
| ----------------- | --------------------------------------------- | ----------------------------------------------------- |
| `learning`        | Reusable technical knowledge                  | "Rust's `#[non_exhaustive]` prevents external match"  |
| `project_context` | Facts tied to a specific project directory    | "This repo uses a custom Swift toolchain at /opt/..." |
| `gotcha`          | Pitfalls, footguns, things that surprised you | "macOS 14 changed clipsToBounds default for NSView"   |

### How Search Works

Entries are indexed with SQLite FTS5 (full-text search). When Claude searches, it matches against title, content, technology, and tags — ranked by relevance. Searches can be filtered by type, technology, or project path.

### Where Data Lives

The database is created automatically at `~/.config/claude-brain/brain.db`.

Override with the `BRAIN_DB_PATH` environment variable:

```bash
claude mcp add --scope user -e BRAIN_DB_PATH=/custom/path/brain.db brain -- /path/to/claude-brain
```

## Tools Reference

### Write

| Tool           | Description                                             |
| -------------- | ------------------------------------------------------- |
| `store_entry`  | Store a new entry (deduplicates by title+type+project)  |
| `update_entry` | Partially update an entry (only provided fields change) |
| `delete_entry` | Delete an entry by ID                                   |

### Read

| Tool                  | Description                                              |
| --------------------- | -------------------------------------------------------- |
| `search_entries`      | Full-text search with optional type/tech/project filters |
| `get_entry`           | Get a single entry by ID                                 |
| `get_project_context` | Get all entries scoped to a project directory            |
| `list_entries`        | Paginated list with filters (type, technology, tags)     |

### Utility

| Tool                | Description                                 |
| ------------------- | ------------------------------------------- |
| `stats`             | Total count, counts by type, recent entries |
| `list_technologies` | All distinct technologies stored            |
| `list_tags`         | All distinct tags in use                    |

## Teaching Claude to Use It

To get the most out of claude-brain, add instructions to your `CLAUDE.md` (global or per-project) telling Claude when and how to use it:

```markdown
## Knowledge Management

You have access to a persistent knowledge store via the `brain` MCP server.

### When to store

- When you discover a non-obvious behavior, gotcha, or workaround
- When the user shares project-specific context that should persist
- When you learn something reusable about a technology or pattern

### When to search

- At the start of a session, search for project context: `get_project_context`
- Before debugging, search for known gotchas: `search_entries` with the relevant technology
- When the user asks "do you remember..." or "what do you know about..."

### Guidelines

- Keep entries concise and actionable — future you reads these in a context window
- Use specific titles: "WKWebView needs network entitlement in sandbox" not "WebView issue"
- Tag entries for discoverability: technology, framework, topic
- Use `project_context` type with the project path for repo-specific facts
```

## Troubleshooting

Logs go to stderr (not stdout — stdout is reserved for the MCP protocol). Set `RUST_LOG=debug` for verbose output.
