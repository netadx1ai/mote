# Master Plan: Tiny Notion

A lightweight, offline-first Notion clone built entirely in Rust with Dioxus. Ultra-simple, clean UI. Stores data as flat markdown files + SQLite metadata, with optional GitHub sync.

## Design Principles

- **Simple over clever** — minimal abstractions, no over-engineering
- **All-Rust** — single binary, no JS/TS/Node, no webview bridge overhead
- **Offline-first** — everything works without internet, GitHub sync is optional
- **File-friendly** — documents stored as `.md` files, git-friendly by design
- **Fast** — SQLite FTS5 for full-text search, SQLite for structured data, flat files for content

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI framework | Dioxus 0.7 (desktop) |
| Structured data | SQLite (via rusqlite, bundled) |
| Full-text search | SQLite FTS5 |
| Content storage | Flat `.md` files |
| Markdown rendering | pulldown-cmark |
| Sync | GitHub (planned - Spec 5) |

## Specs (Execution Order)

| # | Spec | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | `01-scaffold` | Dioxus project setup, Rust app skeleton, build pipeline | None | ✅ DONE |
| 2 | `02-storage-layer` | SQLite schema, FTS5 search, flat-file manager | Spec 1 | ✅ DONE |
| 3 | `03-documents` | Markdown editor, document tree/sidebar, notes view | Spec 2 | ✅ DONE |
| 4 | `04-tasks` | Task lists, sub-tasks, status, priority, project grouping | Spec 2 | ✅ DONE |
| 5 | `05-github-sync` | GitHub repo init/clone, push/pull sync, conflict resolution | Spec 2 | ⬜ TODO |
| 6 | `06-ux-overhaul` | macOS 2026 design system, command palette, keyboard shortcuts, status bar, sidebar redesign, toast system, task enhancements, folder picker | Specs 1–4 | ⬜ TODO |

## Architecture

```
tiny-notion/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point
│   ├── models/mod.rs           # Data types (Item, TaskStatus, etc.)
│   ├── storage/
│   │   ├── mod.rs              # Storage facade
│   │   ├── db.rs               # SQLite operations
│   │   ├── files.rs            # Flat file manager
│   │   └── search.rs           # SQLite FTS5 search index
│   └── ui/
│       ├── mod.rs              # UI module declarations
│       ├── app.rs              # App shell, state, welcome screen
│       ├── sidebar.rs          # Sidebar navigation + tree
│       ├── editor.rs           # Markdown editor (edit/split/preview)
│       ├── task_view.rs        # Task/project view with sub-tasks
│       └── markdown.rs         # pulldown-cmark rendering
└── .kiro/specs/                # Design specs
```

## Data Architecture

```
~/my-workspace/                    # user-chosen directory
├── .tiny-notion.db               # SQLite — metadata, tasks, settings
├── .tiny-notion-fts.db           # SQLite FTS5 — full-text search index
├── docs/
│   └── *.md                      # Documents
├── notes/
│   └── YYYY-MM-DD-*.md           # Notes
└── .git/                          # GitHub sync (optional, future)
```

## Evolution Log

1. **v0.1 (initial)** — Tauri 2.0 + SvelteKit + RocksDB
2. **v0.1 (revised)** — Dropped RocksDB for SQLite FTS5 (libclang too heavy)
3. **v0.2 (current)** — Full refactor to Dioxus (all-Rust, no JS/TS)
