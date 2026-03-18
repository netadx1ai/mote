# Requirements: Storage Layer

## Overview
Implement the hybrid storage engine: SQLite for metadata/tasks, flat `.md` files for content, RocksDB for full-text search indexing. Expose via Tauri commands.

## User Stories

### US-1: Create and Read Items
**As a** user, **I want** to create documents, notes, and tasks that persist across restarts, **so that** my data is saved.

**Acceptance Criteria:**
- [ ] AC-1.1: SQLite schema stores item metadata (id, title, type, parent_id, created_at, updated_at)
- [ ] AC-1.2: Document/note content saved as `.md` files in workspace
- [ ] AC-1.3: Tasks stored fully in SQLite (no separate file)
- [ ] AC-1.4: Tauri commands for create/read/update/delete items

### US-2: Search
**As a** user, **I want** to search across all my content, **so that** I can find things quickly.

**Acceptance Criteria:**
- [ ] AC-2.1: RocksDB indexes document/note content for full-text search
- [ ] AC-2.2: Search returns results ranked by relevance
- [ ] AC-2.3: Index updates on content save

### US-3: Tree Structure
**As a** user, **I want** to organize items in a tree (folders, nested docs), **so that** I can structure my workspace.

**Acceptance Criteria:**
- [ ] AC-3.1: Items support parent_id for tree hierarchy
- [ ] AC-3.2: Reordering/moving items updates parent_id and sort_order
- [ ] AC-3.3: Deleting a parent cascades to children (soft delete)

## Out of Scope
- UI for CRUD (Spec 3/4)
- GitHub sync (Spec 5)
