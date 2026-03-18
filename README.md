# Mote

A lightweight, offline-first workspace app built entirely in Rust.

Documents, tasks, notes, and simple project management — stored as markdown files and SQLite.

## Features

- **Documents** — Markdown editor with edit/split/preview modes, rendered with pulldown-cmark
- **Formatting toolbar** — Bold, italic, strikethrough, code, headings, lists, tables, and more
- **Slash commands** — Type `/` to open a searchable command palette (15 block types)
- **Tasks** — Create tasks with status (todo/in-progress/done/cancelled) and priority (low/med/high/urgent)
- **Sub-tasks** — Nest tasks under tasks or projects, see completion progress
- **Projects** — Group tasks into projects with progress bars and status counts, stored as `.md` files
- **Notes** — Quick-create date-stamped markdown notes
- **Built-in browser** — Multi-tab web browser with URL bar, auto-https, and search fallback
- **Tree sidebar** — Navigate docs, tasks, notes, and settings with drag-drop reordering
- **Filesystem sync** — Auto-discovers `.md` files added externally on startup
- **Git auto-sync** — Auto-commits and pushes to GitHub on every save (background, non-blocking)
- **Backup & restore** — Timestamped zip archives of your workspace data
- **Export/import** — Portable JSON export with duplicate-aware import
- **Flat-file storage** — Documents, notes, and projects saved as `.md` files (git-friendly)
- **SQLite metadata** — Item hierarchy, task status/priority stored in SQLite
- **Full-text search** — SQLite FTS5 index
- **Offline-first** — No internet required, everything runs locally

## Tech Stack

| Component | Technology |
|-----------|-----------|
| UI | [Dioxus](https://dioxuslabs.com) 0.7 (desktop) |
| Database | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled) |
| Search | SQLite FTS5 |
| Markdown | [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) |
| Sanitizer | [ammonia](https://github.com/rust-ammonia/ammonia) |
| Content | Flat `.md` files |

Single Rust binary. No JavaScript, no Node.js, no Electron.

## Build

```bash
# Prerequisites: Rust toolchain
cargo build --release
```

The binary is at `target/release/mote`.

## Run

```bash
cargo run
# or
./target/release/mote
```

On first launch, enter a workspace path (e.g. `~/Documents/my-workspace`). The app creates a `mote-data/` directory with `docs/`, `notes/`, and `projects/` subdirectories and a SQLite database.

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
└── mote-data/                # git repo (auto-initialized)
    ├── .git/
    ├── .gitignore            # Excludes SQLite temp files
    ├── .mote.db              # SQLite metadata + FTS index
    ├── docs/
    │   └── my-document.md
    ├── notes/
    │   └── 2026-03-18-note.md
    └── projects/
        └── my-project.md
```

All Mote data lives inside `mote-data/`, keeping the workspace root clean. Documents, notes, and projects are plain markdown files — edit them in any editor, version them with git. Files added externally are auto-discovered on startup.

### Git Sync Setup

The app auto-initializes a git repo in `mote-data/` and commits on every save. To enable push to GitHub:

```bash
cd ~/my-workspace/mote-data
git remote add origin https://github.com/you/your-repo.git
git push -u origin main
```

After that, every create/edit/delete auto-pushes in the background.

## Roadmap

- [ ] Keyboard shortcuts
- [ ] Search UI
- [ ] Kanban board view for tasks

## License

MIT
