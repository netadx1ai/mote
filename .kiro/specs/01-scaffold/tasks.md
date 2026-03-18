# Tasks: Project Scaffold

> Generated from [requirements.md](./requirements.md) and [design.md](./design.md).

## Task List

### Phase 1: Project Init

- [ ] **Task 1:** Initialize Tauri 2.0 + SvelteKit project with `cargo tauri init`
  - Satisfies: AC-1.1
  - Dependencies: None

- [ ] **Task 2:** Configure SvelteKit with static adapter (SPA mode)
  - File(s): `svelte.config.js`, `vite.config.ts`
  - Satisfies: AC-1.1
  - Dependencies: Task 1

- [ ] **Task 3:** Set up Rust backend skeleton with AppState and health command
  - File(s): `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs`
  - Satisfies: AC-1.3
  - Dependencies: Task 1

### Phase 2: App Shell UI

- [ ] **Task 4:** Create app shell layout (sidebar + content area)
  - File(s): `src/routes/+layout.svelte`, `src/app.css`
  - Satisfies: AC-1.2
  - Dependencies: Task 2

- [ ] **Task 5:** Add sidebar navigation (Docs, Tasks, Notes, Settings sections)
  - File(s): `src/lib/components/Sidebar.svelte`
  - Satisfies: AC-1.2
  - Dependencies: Task 4

- [ ] **Task 6:** Create placeholder pages for each section
  - File(s): `src/routes/docs/+page.svelte`, `src/routes/tasks/+page.svelte`, etc.
  - Satisfies: AC-1.2
  - Dependencies: Task 4

### Phase 3: Workspace Setup

- [ ] **Task 7:** Implement workspace selection dialog (first launch)
  - File(s): `src/routes/+page.svelte`, `src-tauri/src/commands/mod.rs`
  - Satisfies: AC-2.1, AC-2.2, AC-2.3
  - Dependencies: Task 3, Task 4

## Progress Summary

| Phase | Total | Done | Remaining |
|-------|-------|------|-----------|
| Phase 1 | 3 | 0 | 3 |
| Phase 2 | 3 | 0 | 3 |
| Phase 3 | 1 | 0 | 1 |
| **Total** | **7** | **0** | **7** |
