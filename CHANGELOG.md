# Changelog

## [0.8.0] - 2026-03-18

### Added
- **Project `.md` files** — projects now stored as markdown files in `projects/`, same as docs and notes
- **Filesystem sync** — auto-discovers `.md` files added externally to `docs/`, `notes/`, and `projects/` on startup
  - Extracts title from first `# heading` (falls back to filename)
  - Runs on app launch and workspace switch
- **`mote-data/` data directory** — all DB and files now live under `workspace/mote-data/`, keeping the workspace root clean
  - Auto-migration: legacy `.mote.db`, `docs/`, `notes/` at root are moved into `mote-data/`
- **`bundle-macos.sh`** — build + deploy macOS `.app` bundle in one command

### Fixed
- **Sidebar drag-drop rewrite** — completely reworked for smooth reordering
  - Invisible overlay drop zones (top/bottom halves for items, top/mid/bottom for containers)
  - Proper sort_order via `reorder_siblings()` — renumbers all siblings (0, 100, 200...)
  - Drop indicators use `box-shadow: inset` instead of `border + margin` (no layout shift)
  - Replaced global `Mutex` with reactive `Signal` for drag tracking (no flickering)
  - Dragged item shows reduced opacity for visual feedback

### Changed
- Backup/restore now targets `mote-data/` directory specifically
- Updated README with current features, rebranded references, correct workspace structure

## [0.7.0] - 2026-03-18

### Added
- **Claude Code skill** — `mote-dev` skill for AI-assisted development

## [0.6.0] - 2026-03-18

### Fixed
- **Doc saving bug** — content was only saved on blur, lost when clicking sidebar (component unmount). Now saves pending content when navigating away + tracks dirty state

### Added
- **Built-in web browser** — new "Web" tab in sidebar
  - Multiple tabs with +/close buttons
  - URL bar with Enter-to-navigate
  - Auto-adds `https://` and falls back to DuckDuckGo search
  - Tab title auto-updates from hostname
  - Reload button
  - Rendered via iframe in the webview

## [0.5.0] - 2026-03-18 — Rebranded to **Mote**

### Rebrand
- **New name: Mote** — short, memorable, implies lightweight/minimal
- **New icon** — dark rounded rect with glowing blue/purple mote + three particle dots
- **macOS bundle** — `/Applications/Mote.app` with proper Info.plist and icns icon

### UI Overhaul (Notion-inspired dark theme)
- **Color palette** — true dark (#191919 bg, #202020 sidebar) matching Notion dark mode, away from the navy-blue scheme
- **Sidebar** — Notion-style: subtle hover reveals for action buttons, tighter spacing, softer typography, no visible borders
- **Content area** — centered max-width (900px), generous padding, breathing room
- **Title input** — large 2.2em bold heading like Notion page titles
- **Typography** — Inter/SF Pro system fonts, better line-height, letter-spacing tuned
- **Hover reveals** — delete/status buttons only appear on hover (less visual clutter)
- **Task items** — cleaner layout, hover highlight, done state more subtle
- **Editor textarea** — borderless, blends with background
- **Preview** — better code highlighting (#e06c75 for inline code), softer blockquotes
- **Settings** — 3-column stat grid, cleaner section cards
- **Slash menu** — refined popup with shadow, better selected state
- **Scrollbars** — thin, subtle, Notion-style
- **CSS extracted** — moved from inline Rust string to `styles.css` file (`include_str!`)

## [0.4.0] - 2026-03-18

### Added
- **Formatting toolbar** — Bold, Italic, Strikethrough, Code, Link, Heading, List, Task List, Code Block, Table, Quote, Divider buttons
- **Slash commands** — type `/` in editor to open a searchable command palette (15 block types)
  - Navigate with arrow keys, Enter to insert, Escape to close
  - Auto-triggers when typing `/` at line start or after whitespace
  - Filterable: type to narrow results
- **Word/line count** in toolbar status area
- **Block insert buttons** in toolbar for quick markdown structure insertion

### Security Fixes
- **Critical: Zip slip** — validate all zip entry paths, reject `..` traversal, verify paths stay within workspace
- **High: XSS prevention** — added `ammonia` HTML sanitizer for markdown preview output
- **High: Path traversal** — `FileManager.safe_resolve()` rejects `..` and verifies canonical paths
- **Medium: FTS5 injection** — user queries quoted to prevent operator injection
- **Medium: Zip bomb protection** — 1GB decompressed size limit, 50K entry limit
- **Medium: Symlink safety** — backup skips symlinks to prevent exfiltrating data outside workspace

### Code Quality (Simplify)
- Extracted shared helpers: `open_workspace()`, `with_storage()`, `update_item_field()`
- Eliminated ~150 lines of duplicated code across 8+ locations
- Merged 2 SQLite databases into 1 (FTS5 in same DB)
- Replaced serde_json enum hack with `as_str()`/`FromStr` (zero-alloc)
- Replaced stringly-typed task filter with `TaskFilter` enum
- `UpdateItemRequest` implements `Default` for cleaner call sites
- `Item::db_content()` single source of truth for DB storage logic
- TreeNode uses pre-computed `children_map` (was O(N^2), now O(N))
- Single-pass stats computation in settings
- Markdown skipped in edit-only mode
- Fixed: soft delete now cascades recursively via CTE
- Fixed: search returns correct ItemType (was hardcoded to Document)
- Fixed: slugify collapses consecutive dashes

## [0.3.0] - 2026-03-18

### Added
- **Backup** — create timestamped zip archives of entire workspace (DBs + markdown files)
- **Restore** — restore from a zip backup, replaces current workspace data (preserves `.git`)
- **Export JSON** — export all items with content as a portable JSON file
- **Import JSON** — merge items from a JSON export into current workspace (skip duplicates)
- **Settings view** — new "Cfg" tab in sidebar with:
  - Workspace path display and switch workspace
  - Workspace statistics (total items, docs, tasks, notes, projects, done count)
  - Backup/restore controls with path inputs
  - Export/import JSON controls
  - Status messages for all operations (success/error feedback)

### Changed
- Sidebar now has 4 tabs: Docs, Tasks, Notes, Cfg

## [0.2.0] - 2026-03-18

### Changed
- **Full refactor from Tauri+SvelteKit to Dioxus** — the entire app is now pure Rust
  - Removed: Tauri, SvelteKit, Node.js, npm, all JavaScript/TypeScript
  - Added: Dioxus 0.7 desktop framework
  - Result: single Rust binary, faster startup, simpler build
- Replaced `marked` + `mermaid.js` with `pulldown-cmark` for server-side markdown rendering

### Retained
- SQLite storage layer (rusqlite, bundled)
- SQLite FTS5 full-text search index
- Flat `.md` file storage for documents and notes
- All data models (Item, TaskStatus, TaskPriority, etc.)
- Workspace config persistence (~/.config/tiny-notion/)

## [0.1.0] - 2026-03-18

### Added
- Initial project scaffold with Tauri 2.0 + SvelteKit 5
- SQLite metadata storage with tree hierarchy (parent_id, sort_order)
- SQLite FTS5 full-text search index
- Flat-file `.md` storage for documents and notes
- Markdown editor with edit/split/preview modes
- Mermaid diagram rendering in preview
- Task management with status (todo/in_progress/done/cancelled)
- Task priority levels (none/low/medium/high/urgent)
- Sub-tasks with parent-child nesting
- Project grouping with progress bars
- Notes with auto-generated date titles
- Tree sidebar navigation with section tabs (Docs/Tasks/Notes)
- Workspace selection on first launch
- Dark theme UI
- Auto-save with 1s debounce

### Architecture Decisions
- Dropped RocksDB in favor of SQLite FTS5 (avoids libclang build dependency)
- SvelteKit in SPA/SSG mode with static adapter
- Tauri 2.0 with dialog and fs plugins
