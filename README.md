# Tiny Notion

A lightweight, offline-first Notion clone built entirely in Rust.

Documents, tasks, notes, and simple project management — stored as markdown files and SQLite.

## Features

- **Documents** — Markdown editor with edit/split/preview modes, rendered with pulldown-cmark
- **Tasks** — Create tasks with status (todo/in-progress/done/cancelled) and priority (low/med/high/urgent)
- **Sub-tasks** — Nest tasks under tasks or projects, see completion progress
- **Projects** — Group tasks into projects with progress bars and status counts
- **Notes** — Quick-create date-stamped markdown notes
- **Tree sidebar** — Navigate docs, tasks, and notes in a collapsible tree
- **Flat-file storage** — Documents and notes saved as `.md` files (git-friendly)
- **SQLite metadata** — Item hierarchy, task status/priority stored in SQLite
- **Full-text search** — SQLite FTS5 index (planned UI integration)
- **Offline-first** — No internet required, everything runs locally

## Tech Stack

| Component | Technology |
|-----------|-----------|
| UI | [Dioxus](https://dioxuslabs.com) 0.7 (desktop) |
| Database | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled) |
| Search | SQLite FTS5 |
| Markdown | [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) |
| Content | Flat `.md` files |

Single Rust binary. No JavaScript, no Node.js, no Electron.

## Build

```bash
# Prerequisites: Rust toolchain
cargo build --release
```

The binary is at `target/release/tiny-notion`.

## Run

```bash
cargo run
# or
./target/release/tiny-notion
```

On first launch, enter a workspace path (e.g. `~/Documents/my-workspace`). The app creates `docs/` and `notes/` directories and SQLite databases in that folder.

## Dev

```bash
# Install dx CLI (optional, for hot-reload)
cargo install dioxus-cli

# Run with hot-reload
dx serve

# Or plain cargo
cargo run
```

## Workspace Structure

```
~/my-workspace/
├── .tiny-notion.db           # SQLite metadata
├── .tiny-notion-fts.db       # Full-text search index
├── docs/
│   └── my-document.md
└── notes/
    └── 2026-03-18-note.md
```

Documents and notes are plain markdown files — edit them in any editor, version them with git.

## Roadmap

- [ ] GitHub sync (push/pull workspace to a repo)
- [ ] Drag-and-drop reordering
- [ ] Keyboard shortcuts
- [ ] Search UI
- [ ] Mermaid diagram rendering (via JS interop or SVG generation)
- [ ] Kanban board view for tasks

## License

MIT
