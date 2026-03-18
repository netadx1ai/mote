# Design: Project Scaffold

## Overview
Standard Tauri 2.0 + SvelteKit setup. SvelteKit runs in SSG/SPA mode (static adapter). Rust backend exposes Tauri commands. App shell has sidebar + content area layout.

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   Tauri Shell                     │
│  ┌──────────────────────────────────────────┐    │
│  │           SvelteKit Frontend              │    │
│  │  ┌──────────┐  ┌─────────────────────┐   │    │
│  │  │ Sidebar   │  │   Content Area      │   │    │
│  │  │ - Docs    │  │   (router outlet)   │   │    │
│  │  │ - Tasks   │  │                     │   │    │
│  │  │ - Notes   │  │                     │   │    │
│  │  │ - Settings│  │                     │   │    │
│  │  └──────────┘  └─────────────────────┘   │    │
│  └──────────────────────────────────────────┘    │
│  ┌──────────────────────────────────────────┐    │
│  │           Rust Backend                    │    │
│  │  - AppState (workspace path, DB handles)  │    │
│  │  - Tauri commands                         │    │
│  └──────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

---

## Project Structure

```
tiny-notion/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs           # Tauri entry point
│   │   ├── lib.rs            # App setup, state init
│   │   ├── commands/
│   │   │   └── mod.rs        # Tauri command handlers
│   │   ├── storage/
│   │   │   └── mod.rs        # Storage engine (Spec 2)
│   │   └── models/
│   │       └── mod.rs        # Data types
│   └── icons/
├── src/
│   ├── routes/
│   │   ├── +layout.svelte    # App shell (sidebar + content)
│   │   ├── +page.svelte      # Home/welcome
│   │   ├── docs/
│   │   ├── tasks/
│   │   ├── notes/
│   │   └── settings/
│   ├── lib/
│   │   ├── components/       # Shared UI components
│   │   ├── stores/           # Svelte stores
│   │   └── tauri.ts          # Tauri invoke wrappers
│   └── app.html
├── static/
├── svelte.config.js
├── vite.config.ts
├── package.json
└── .kiro/
```

---

## Key Decisions

- **SvelteKit adapter**: `@sveltejs/adapter-static` with `fallback: 'index.html'` for SPA mode
- **Styling**: Plain CSS with CSS custom properties for theming (no framework)
- **Workspace config**: Stored in Tauri's app data dir (`app_config_dir()`)
