# Requirements: Documents & Notes UI

## Overview
Markdown editor with live preview, Mermaid diagram support, document tree sidebar, and notes view.

## User Stories

### US-1: Markdown Editing
**As a** user, **I want** to write and edit markdown documents, **so that** I can create rich content.

**Acceptance Criteria:**
- [ ] AC-1.1: Split or toggle editor with live preview
- [ ] AC-1.2: Standard markdown syntax supported (headers, lists, code blocks, tables, links, images)
- [ ] AC-1.3: Auto-save on edit (debounced 1s)
- [ ] AC-1.4: Keyboard shortcuts for common formatting (bold, italic, code)

### US-2: Mermaid Diagrams
**As a** user, **I want** to embed Mermaid diagrams in markdown, **so that** I can visualize flows and charts.

**Acceptance Criteria:**
- [ ] AC-2.1: ```mermaid code blocks render as diagrams in preview
- [ ] AC-2.2: Supports flowchart, sequence, gantt, and class diagrams

### US-3: Document Tree
**As a** user, **I want** a sidebar showing my document hierarchy, **so that** I can navigate my workspace.

**Acceptance Criteria:**
- [ ] AC-3.1: Sidebar shows tree of documents and folders
- [ ] AC-3.2: Click to open, right-click for rename/delete/new child
- [ ] AC-3.3: Drag-and-drop reordering
- [ ] AC-3.4: Collapsible folders

### US-4: Notes
**As a** user, **I want** a quick-notes section for short-form content, **so that** I can jot things down fast.

**Acceptance Criteria:**
- [ ] AC-4.1: Notes section in sidebar separate from documents
- [ ] AC-4.2: Quick-create note with auto-generated date title
- [ ] AC-4.3: Notes use same markdown editor as documents

## Out of Scope
- WYSIWYG block editor (keep it simple — markdown source + preview)
- Image upload/management
- Collaborative editing
