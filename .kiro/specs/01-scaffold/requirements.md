# Requirements: Project Scaffold

## Overview
Set up the Tauri 2.0 + SvelteKit project with Rust backend skeleton, build pipeline, and basic app shell.

## User Stories

### US-1: App Launches
**As a** user, **I want** to launch Tiny Notion as a desktop app, **so that** I can start using it.

**Acceptance Criteria:**
- [ ] AC-1.1: `cargo tauri dev` starts the app with a SvelteKit frontend
- [ ] AC-1.2: App window opens with a sidebar + main content layout
- [ ] AC-1.3: Rust backend initializes and responds to a health-check Tauri command

### US-2: Workspace Selection
**As a** user, **I want** to choose a workspace directory on first launch, **so that** my data is stored where I want.

**Acceptance Criteria:**
- [ ] AC-2.1: First launch shows a "Choose Workspace" dialog
- [ ] AC-2.2: Selected path is persisted in app config
- [ ] AC-2.3: Subsequent launches open the last workspace automatically

## Out of Scope
- Actual document/task CRUD (Spec 2+)
- GitHub integration (Spec 5)
- Themes or customization
