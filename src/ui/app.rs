use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::models::*;
use crate::storage::Storage;
use crate::ui::browser::BrowserView;
use crate::ui::editor::Editor;
use crate::ui::settings::Settings;
use crate::ui::sidebar::Sidebar;
use crate::ui::task_view::TaskView;

#[derive(Clone)]
pub struct AppState {
    pub storage: Option<Arc<Storage>>,
    pub workspace_path: Option<PathBuf>,
    pub tree: Vec<Item>,
    pub active_item: Option<Item>,
    pub active_section: Section,
}

#[derive(Clone, PartialEq)]
pub enum Section {
    Docs,
    Tasks,
    Notes,
    Browser,
    Settings,
}

fn config_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mote")
        .join("config.json")
}

impl AppState {
    fn new() -> Self {
        let config_path = config_file_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let saved: Option<WorkspaceConfig> = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

        let workspace_path = saved.and_then(|c| c.workspace_path.map(PathBuf::from));

        if let Some(ref wp) = workspace_path {
            if let Ok(storage) = Storage::new(wp) {
                let tree = storage.get_tree().unwrap_or_default();
                return AppState {
                    storage: Some(Arc::new(storage)),
                    workspace_path: Some(wp.clone()),
                    tree,
                    active_item: None,
                    active_section: Section::Docs,
                };
            }
        }

        AppState {
            storage: None,
            workspace_path: None,
            tree: vec![],
            active_item: None,
            active_section: Section::Docs,
        }
    }

    pub fn save_config(&self) {
        let config = WorkspaceConfig {
            workspace_path: self.workspace_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        };
        let path = config_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, serde_json::to_string_pretty(&config).unwrap());
    }

    /// Pre-computed children map: parent_id -> Vec<Item>
    pub fn children_map(&self) -> HashMap<Option<String>, Vec<Item>> {
        let mut map: HashMap<Option<String>, Vec<Item>> = HashMap::new();
        for item in &self.tree {
            if !item.deleted {
                map.entry(item.parent_id.clone()).or_default().push(item.clone());
            }
        }
        // Sort each group
        for children in map.values_mut() {
            children.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then(a.created_at.cmp(&b.created_at)));
        }
        map
    }
}

// --- Shared UI helpers ---

/// Open a workspace path: create Storage, refresh tree, save config.
pub fn open_workspace(mut state: Signal<AppState>, path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    let storage = Storage::new(&path)?;
    let _ = storage.files.ensure_dirs();
    let tree = storage.get_tree().unwrap_or_default();
    let mut st = state.write();
    st.storage = Some(Arc::new(storage));
    st.workspace_path = Some(path);
    st.tree = tree;
    st.active_item = None;
    st.save_config();
    Ok(())
}

/// Run a mutation with storage: clone Arc, run closure, refresh tree.
pub fn with_storage<F, R>(mut state: Signal<AppState>, f: F) -> Result<R, String>
where
    F: FnOnce(&Storage) -> Result<R, String>,
{
    let st = state.read();
    let storage = st.storage.as_ref().ok_or("No workspace configured")?.clone();
    drop(st);
    let result = f(&storage)?;
    let tree = storage.get_tree().unwrap_or_default();
    state.write().tree = tree;
    Ok(result)
}

/// Update an item field and refresh state.
pub fn update_item_field(
    mut state: Signal<AppState>,
    id: &str,
    title: Option<String>,
    content: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
) {
    let id = id.to_string();
    let _ = with_storage(state, |storage| {
        let updated = storage.update_item(UpdateItemRequest {
            id: id.clone(),
            title,
            content,
            status,
            priority,
            ..Default::default()
        })?;
        let st_active_id = state.read().active_item.as_ref().map(|i| i.id.clone());
        if st_active_id.as_deref() == Some(&id) {
            state.write().active_item = Some(updated);
        }
        Ok(())
    });
}

/// Global pending editor content — sidebar reads this before switching items.
/// (item_id, content, is_dirty)
pub static EDITOR_PENDING: std::sync::Mutex<Option<(String, String)>> = std::sync::Mutex::new(None);

/// Called by editor on every keystroke to register pending unsaved content.
pub fn set_editor_pending(id: &str, content: &str) {
    if let Ok(mut pending) = EDITOR_PENDING.lock() {
        *pending = Some((id.to_string(), content.to_string()));
    }
}

/// Called by editor on successful save to clear pending.
pub fn clear_editor_pending() {
    if let Ok(mut pending) = EDITOR_PENDING.lock() {
        *pending = None;
    }
}

/// Flush any pending editor content to storage. Called by sidebar BEFORE switching items.
pub fn flush_editor_pending(state: Signal<AppState>) {
    let pending = EDITOR_PENDING.lock().ok().and_then(|mut p| p.take());
    if let Some((id, content)) = pending {
        let st = state.read();
        if let Some(ref storage) = st.storage {
            let storage = storage.clone();
            drop(st);
            let _ = storage.update_item(UpdateItemRequest {
                id,
                content: Some(content),
                ..Default::default()
            });
        }
    }
}

const STYLES: &str = include_str!("styles.css");

#[component]
pub fn App() -> Element {
    let state = use_signal(|| AppState::new());
    let has_workspace = state.read().storage.is_some();

    rsx! {
        style { {STYLES} }
        if has_workspace {
            div { class: "app-shell",
                Sidebar { state }
                main { class: "content",
                    {
                        let st = state.read();
                        let section = st.active_section.clone();
                        if section == Section::Settings {
                            drop(st);
                            rsx! { Settings { state } }
                        } else if section == Section::Browser {
                            drop(st);
                            rsx! { BrowserView {} }
                        } else if let Some(ref item) = st.active_item {
                            let item_clone = item.clone();
                            drop(st);
                            match item_clone.item_type {
                                ItemType::Task | ItemType::Project => rsx! {
                                    TaskView { state, item: item_clone }
                                },
                                _ => rsx! {
                                    Editor { state, item: item_clone }
                                },
                            }
                        } else {
                            drop(st);
                            rsx! {
                                div { class: "empty-state",
                                    h2 { "Welcome to your workspace" }
                                    p { "Select an item from the sidebar or create a new one." }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            WelcomeScreen { state }
        }
    }
}

#[component]
fn WelcomeScreen(state: Signal<AppState>) -> Element {
    let mut path_input = use_signal(|| String::new());

    let do_open = move |_: ()| {
        let path_str = path_input.read().clone();
        if !path_str.is_empty() {
            if let Err(e) = open_workspace(state, PathBuf::from(&path_str)) {
                eprintln!("Failed to open workspace: {e}");
            }
        }
    };

    rsx! {
        div { class: "welcome",
            div { class: "welcome-card",
                h1 { "Mote" }
                p { "Your lightweight workspace for docs, tasks, and notes." }
                div { class: "workspace-input-row",
                    input {
                        class: "workspace-input",
                        placeholder: "Enter workspace path (e.g. ~/Documents/my-workspace)",
                        value: "{path_input}",
                        oninput: move |e| path_input.set(e.value()),
                        onkeypress: move |e| {
                            if e.key() == Key::Enter { do_open(()); }
                        },
                    }
                    button { class: "btn-primary", onclick: move |_| do_open(()), "Open" }
                }
                p { class: "hint", "Enter a folder path. Files will be stored as markdown." }
            }
        }
    }
}
