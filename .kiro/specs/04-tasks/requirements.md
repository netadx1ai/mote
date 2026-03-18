# Requirements: Tasks & Project Management

## Overview
Task management with sub-tasks, status tracking, priority, and project grouping. Simple project management view.

## User Stories

### US-1: Task CRUD
**As a** user, **I want** to create, edit, and delete tasks, **so that** I can track my work.

**Acceptance Criteria:**
- [ ] AC-1.1: Create task with title, optional description (markdown), status, priority
- [ ] AC-1.2: Status options: todo, in_progress, done, cancelled
- [ ] AC-1.3: Priority options: none, low, medium, high, urgent
- [ ] AC-1.4: Inline editing of task title and status

### US-2: Sub-tasks
**As a** user, **I want** to break tasks into sub-tasks, **so that** I can track granular progress.

**Acceptance Criteria:**
- [ ] AC-2.1: Tasks can have child tasks (unlimited nesting)
- [ ] AC-2.2: Parent task shows completion percentage based on children
- [ ] AC-2.3: Indent/outdent to change task hierarchy

### US-3: Project Grouping
**As a** user, **I want** to group tasks into projects, **so that** I can organize work by context.

**Acceptance Criteria:**
- [ ] AC-3.1: "Projects" appear in sidebar as top-level items
- [ ] AC-3.2: Each project has its own task list
- [ ] AC-3.3: Project overview shows task counts by status

### US-4: Task Views
**As a** user, **I want** different views of my tasks, **so that** I can see what matters.

**Acceptance Criteria:**
- [ ] AC-4.1: List view (default) — flat list with filters
- [ ] AC-4.2: Filter by status, priority, project
- [ ] AC-4.3: Sort by created date, priority, or status

## Out of Scope
- Kanban board view (future enhancement)
- Due dates and calendar view
- Assignments (single-user app)
