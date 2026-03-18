use dioxus::prelude::*;
use std::collections::HashMap;

use crate::models::*;
use crate::ui::app::{flush_editor_pending, with_storage, update_item_field, AppState, Section};

/// Global drag state for sidebar
static SIDEBAR_DRAG: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn set_drag(id: Option<&str>) {
    if let Ok(mut d) = SIDEBAR_DRAG.lock() { *d = id.map(|s| s.to_string()); }
}
fn get_drag() -> Option<String> {
    SIDEBAR_DRAG.lock().ok().and_then(|d| d.clone())
}

/// Drop position relative to a tree node
#[derive(Clone, PartialEq)]
enum DropPos {
    Above(String),  // Insert before this item (reorder)
    Into(String),   // Move into this item as child (reparent)
    Below(String),  // Insert after this item (reorder)
}

#[component]
pub fn Sidebar(state: Signal<AppState>) -> Element {
    let section = state.read().active_section.clone();
    let children_map = state.read().children_map();
    let root_items = children_map.get(&None).cloned().unwrap_or_default();
    let mut drop_pos = use_signal(|| Option::<DropPos>::None);

    let docs: Vec<Item> = root_items.iter()
        .filter(|i| i.item_type == ItemType::Document || i.item_type == ItemType::Folder)
        .cloned().collect();
    let tasks: Vec<Item> = root_items.iter()
        .filter(|i| i.item_type == ItemType::Task || i.item_type == ItemType::Project)
        .cloned().collect();
    let notes: Vec<Item> = root_items.iter()
        .filter(|i| i.item_type == ItemType::Note)
        .cloned().collect();

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-header",
                div { class: "logo", "M" }
                span { class: "app-name", "Mote" }
            }
            nav { class: "sections",
                button { class: if section == Section::Docs { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Docs, "Docs" }
                button { class: if section == Section::Tasks { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Tasks, "Tasks" }
                button { class: if section == Section::Notes { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Notes, "Notes" }
                button { class: if section == Section::Browser { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Browser, "Web" }
                button { class: if section == Section::Settings { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Settings, "Cfg" }
            }
            div { class: "tree-container",
                match section {
                    Section::Docs => rsx! {
                        div { class: "section-header",
                            span { "Documents" }
                            button { class: "add-btn", onclick: move |_| create_item(state, ItemType::Document), "+" }
                        }
                        for (idx, item) in docs.iter().enumerate() {
                            TreeNode { state, item: item.clone(), depth: 0, idx: idx as i32, children_map: children_map.clone(), drop_pos }
                        }
                        if docs.is_empty() { p { class: "empty", "No documents yet" } }
                    },
                    Section::Tasks => rsx! {
                        div { class: "section-header",
                            span { "Projects & Tasks" }
                            div { style: "display: flex; gap: 4px;",
                                button { class: "add-btn", title: "New project", onclick: move |_| create_item(state, ItemType::Project), "P+" }
                                button { class: "add-btn", title: "New task", onclick: move |_| create_item(state, ItemType::Task), "T+" }
                            }
                        }
                        for (idx, item) in tasks.iter().enumerate() {
                            TreeNode { state, item: item.clone(), depth: 0, idx: idx as i32, children_map: children_map.clone(), drop_pos }
                        }
                        if tasks.is_empty() { p { class: "empty", "No tasks yet" } }
                    },
                    Section::Notes => rsx! {
                        div { class: "section-header",
                            span { "Notes" }
                            button { class: "add-btn", onclick: move |_| create_item(state, ItemType::Note), "+" }
                        }
                        for (idx, item) in notes.iter().enumerate() {
                            TreeNode { state, item: item.clone(), depth: 0, idx: idx as i32, children_map: children_map.clone(), drop_pos }
                        }
                        if notes.is_empty() { p { class: "empty", "No notes yet" } }
                    },
                    Section::Browser => rsx! {
                        div { class: "section-header", span { "Browser" } }
                        p { class: "empty", "Browse the web inside Mote" }
                    },
                    Section::Settings => rsx! {
                        div { class: "section-header", span { "Settings" } }
                        p { class: "empty", "Configure workspace, backup & restore" }
                    },
                }
            }
        }
    }
}

fn create_item(mut state: Signal<AppState>, item_type: ItemType) {
    let (title, content, status, priority) = match item_type {
        ItemType::Document => ("Untitled".to_string(), Some("# Untitled\n\n".to_string()), None, None),
        ItemType::Note => {
            let title = format!("{} Note", chrono::Utc::now().format("%Y-%m-%d"));
            let content = format!("# {title}\n\n");
            (title, Some(content), None, None)
        }
        ItemType::Task => ("New Task".to_string(), None, Some(TaskStatus::Todo), Some(TaskPriority::None)),
        ItemType::Project => ("New Project".to_string(), None, None, None),
        ItemType::Folder => ("New Folder".to_string(), None, None, None),
    };
    let _ = with_storage(state, |storage| {
        let item = storage.create_item(CreateItemRequest {
            title, item_type, parent_id: None, content, status, priority,
        })?;
        state.write().active_item = Some(item);
        Ok(())
    });
}

fn can_accept_children(item_type: &ItemType) -> bool {
    matches!(item_type, ItemType::Project | ItemType::Folder)
}

#[component]
fn TreeNode(
    state: Signal<AppState>,
    item: Item,
    depth: i32,
    idx: i32,
    children_map: HashMap<Option<String>, Vec<Item>>,
    mut drop_pos: Signal<Option<DropPos>>,
) -> Element {
    let active_id = state.read().active_item.as_ref().map(|i| i.id.clone());
    let is_active = active_id.as_deref() == Some(&item.id);
    let children = children_map.get(&Some(item.id.clone())).cloned().unwrap_or_default();
    let has_children = !children.is_empty();
    let is_container = can_accept_children(&item.item_type);
    let is_task = item.item_type == ItemType::Task;

    // Check drop indicator state
    let dp = drop_pos.read().clone();
    let show_above = matches!(&dp, Some(DropPos::Above(id)) if id == &item.id);
    let show_into = matches!(&dp, Some(DropPos::Into(id)) if id == &item.id);
    let show_below = matches!(&dp, Some(DropPos::Below(id)) if id == &item.id);

    let icon = match item.item_type {
        ItemType::Folder => "F",
        ItemType::Document => "D",
        ItemType::Note => "N",
        ItemType::Project => "P",
        ItemType::Task => item.status.as_ref().map(|s| s.icon()).unwrap_or("○"),
    };

    let padding = format!("padding-left: {}px;", 12 + depth * 16);
    let item_id_click = item.id.clone();
    let item_id_drag = item.id.clone();
    let item_id_over = item.id.clone();
    let item_id_drop = item.id.clone();
    let item_parent = item.parent_id.clone();
    let item_type_drop = item.item_type.clone();

    // Status cycle for tasks (click on icon)
    let item_id_status = item.id.clone();
    let task_status = item.status.clone();

    let mut class = String::from("tree-node");
    if is_active { class.push_str(" active"); }
    if show_into { class.push_str(" drop-target-node"); }
    if show_above { class.push_str(" drop-above"); }
    if show_below { class.push_str(" drop-below"); }

    rsx! {
        div {
            class: "{class}",
            style: "{padding}",
            draggable: "true",
            ondragstart: move |_| {
                set_drag(Some(&item_id_drag));
            },
            ondragend: move |_| {
                set_drag(None);
                drop_pos.set(None);
            },
            ondragover: move |e| {
                e.prevent_default();
                let drag_id = get_drag();
                if drag_id.as_deref() == Some(&*item_id_over) { return; }

                // Determine drop zone based on item type:
                // - Containers (Project/Folder): drop INTO them
                // - Others: drop ABOVE (reorder at same level)
                if can_accept_children(&item.item_type) {
                    drop_pos.set(Some(DropPos::Into(item_id_over.clone())));
                } else {
                    drop_pos.set(Some(DropPos::Above(item_id_over.clone())));
                }
            },
            ondragleave: move |_| {
                // Only clear if we're still the target
                let dp = drop_pos.read().clone();
                let dominated = match &dp {
                    Some(DropPos::Above(id) | DropPos::Into(id) | DropPos::Below(id)) => id == &item.id,
                    None => false,
                };
                if dominated { drop_pos.set(None); }
            },
            ondrop: {
                let target_id = item_id_drop.clone();
                let target_parent = item_parent.clone();
                let target_idx = idx;
                let target_type = item_type_drop.clone();
                move |e| {
                    e.stop_propagation();
                    let drag_id = get_drag();
                    if let Some(drag_id) = drag_id {
                        if drag_id == target_id { set_drag(None); drop_pos.set(None); return; }

                        if can_accept_children(&target_type) {
                            // Drop INTO container: reparent
                            let tid = target_id.clone();
                            let _ = with_storage(state, |storage| {
                                storage.move_item(&drag_id, Some(&tid), 0)?;
                                Ok(())
                            });
                        } else {
                            // Drop ABOVE: reorder at same level
                            let new_order = target_idx * 10;
                            let parent = target_parent.clone();
                            let _ = with_storage(state, |storage| {
                                storage.move_item(&drag_id, parent.as_deref(), new_order)?;
                                Ok(())
                            });
                        }
                    }
                    set_drag(None);
                    drop_pos.set(None);
                }
            },
            onclick: move |_| {
                flush_editor_pending(state);
                let _ = with_storage(state, |storage| {
                    let loaded = storage.get_item(&item_id_click)?;
                    state.write().active_item = Some(loaded);
                    Ok(())
                });
            },

            // Task status icon — clickable to cycle status
            if is_task {
                span {
                    class: "node-icon status-click",
                    onclick: move |e| {
                        e.stop_propagation(); // Don't trigger tree node click
                        if let Some(ref st) = task_status {
                            let next = st.next();
                            update_item_field(state, &item_id_status, None, None, Some(next), None);
                        }
                    },
                    "{icon}"
                }
            } else {
                span { class: "node-icon", "{icon}" }
            }
            span { class: "node-title", "{item.title}" }
            if has_children {
                span { style: "margin-left: auto; font-size: 10px; color: rgba(255,255,255,0.2);", "{children.len()}" }
            }
        }
        for (cidx, child) in children.iter().enumerate() {
            TreeNode { state, item: child.clone(), depth: depth + 1, idx: cidx as i32, children_map: children_map.clone(), drop_pos }
        }
    }
}
