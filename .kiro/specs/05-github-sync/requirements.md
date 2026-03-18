# Requirements: GitHub Sync

## Overview
Optional GitHub integration to back up and sync workspace content via a Git repository.

## User Stories

### US-1: Initialize Sync
**As a** user, **I want** to connect my workspace to a GitHub repo, **so that** my content is backed up.

**Acceptance Criteria:**
- [ ] AC-1.1: Settings page to configure GitHub token and repo URL
- [ ] AC-1.2: "Init Sync" creates/clones the repo in workspace
- [ ] AC-1.3: `.gitignore` auto-generated to exclude `.tiny-notion.db`, `.tiny-notion.rocks/`

### US-2: Push/Pull
**As a** user, **I want** to push and pull changes, **so that** I can sync across machines.

**Acceptance Criteria:**
- [ ] AC-2.1: Manual "Sync Now" button in UI
- [ ] AC-2.2: Auto-sync on save (configurable interval, default 5min)
- [ ] AC-2.3: Status indicator shows sync state (synced, pending, error)

### US-3: Conflict Handling
**As a** user, **I want** conflicts to be resolved gracefully, **so that** I don't lose work.

**Acceptance Criteria:**
- [ ] AC-3.1: Last-write-wins for non-conflicting changes
- [ ] AC-3.2: Conflicting files saved as `.conflict` copies for manual review
- [ ] AC-3.3: Notification shown when conflicts occur

## Out of Scope
- Real-time collaboration
- GitHub OAuth flow (use personal access token)
- Multiple remote repos
