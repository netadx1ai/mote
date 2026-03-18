use dioxus::prelude::*;
use std::collections::HashMap;

use crate::models::*;
use crate::storage::ReorderPos;
use crate::ui::app::{flush_editor_pending, with_storage, update_item_field, AppState, Section};

/// Drop position relative to a tree node
#[derive(Clone, PartialEq)]
enum DropPos {
    Above(String),
    Into(String),
    Below(String),
}

#[component]
pub fn Sidebar(state: Signal<AppState>) -> Element {
    let section = state.read().active_section.clone();
    let children_map = state.read().children_map();
    let root_items = children_map.get(&None).cloned().unwrap_or_default();
    let mut drop_pos = use_signal(|| Option::<DropPos>::None);
    let mut drag_id = use_signal(|| Option::<String>::None);

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
                div {
                    class: if section == Section::Docs { "section-btn active" } else { "section-btn" },
                    onclick: move |_| state.write().active_section = Section::Docs,
                    ondragover: move |e| { e.prevent_default(); },
                    ondrop: move |e| { e.stop_propagation(); drag_id.set(None); drop_pos.set(None); convert_to_section(state, ItemType::Document); state.write().active_section = Section::Docs; },
                    "Docs"
                }
                div {
                    class: if section == Section::Tasks { "section-btn active" } else { "section-btn" },
                    onclick: move |_| state.write().active_section = Section::Tasks,
                    ondragover: move |e| { e.prevent_default(); },
                    ondrop: move |e| { e.stop_propagation(); drag_id.set(None); drop_pos.set(None); convert_to_section(state, ItemType::Project); state.write().active_section = Section::Tasks; },
                    "Tasks"
                }
                div {
                    class: if section == Section::Notes { "section-btn active" } else { "section-btn" },
                    onclick: move |_| state.write().active_section = Section::Notes,
                    ondragover: move |e| { e.prevent_default(); },
                    ondrop: move |e| { e.stop_propagation(); drag_id.set(None); drop_pos.set(None); convert_to_section(state, ItemType::Note); state.write().active_section = Section::Notes; },
                    "Notes"
                }
                div { class: if section == Section::Browser { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Browser, "Web" }
                div { class: if section == Section::Settings { "section-btn active" } else { "section-btn" }, onclick: move |_| state.write().active_section = Section::Settings, "Cfg" }
            }
            div { class: "tree-container",
                match section {
                    Section::Docs => rsx! {
                        div { class: "section-header",
                            span { "Documents" }
                            button { class: "add-btn", onclick: move |_| create_item(state, ItemType::Document), "+" }
                        }
                        for item in docs.iter() {
                            TreeNode { state, item: item.clone(), depth: 0, children_map: children_map.clone(), drop_pos, drag_id }
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
                        for item in tasks.iter() {
                            TreeNode { state, item: item.clone(), depth: 0, children_map: children_map.clone(), drop_pos, drag_id }
                        }
                        if tasks.is_empty() { p { class: "empty", "No tasks yet" } }
                    },
                    Section::Notes => rsx! {
                        div { class: "section-header",
                            span { "Notes" }
                            button { class: "add-btn", onclick: move |_| create_item(state, ItemType::Note), "+" }
                        }
                        for item in notes.iter() {
                            TreeNode { state, item: item.clone(), depth: 0, children_map: children_map.clone(), drop_pos, drag_id }
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

/// Global drag ID for cross-section drops (avoids signal re-render issues)
static CROSS_DRAG_ID: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn set_cross_drag(id: Option<&str>) {
    if let Ok(mut d) = CROSS_DRAG_ID.lock() { *d = id.map(|s| s.to_string()); }
}
fn get_cross_drag() -> Option<String> {
    CROSS_DRAG_ID.lock().ok().and_then(|d| d.clone())
}

/// Convert a dragged item to the target section's type.
fn convert_to_section(mut state: Signal<AppState>, target_type: ItemType) {
    if let Some(id) = get_cross_drag() {
        let target = target_type.clone();
        let _ = with_storage(state, |storage| {
            let item = storage.get_item(&id)?;
            if item.item_type != target {
                storage.convert_item_type(&id, target)?;
            }
            Ok(())
        });
        set_cross_drag(None);
    }
}

fn do_drop(
    state: Signal<AppState>,
    mut drag_id: Signal<Option<String>>,
    mut drop_pos: Signal<Option<DropPos>>,
    target_id: &str,
    pos: ReorderPos,
) {
    if let Some(dragged) = drag_id.read().clone() {
        if dragged != target_id {
            let target = target_id.to_string();
            let _ = with_storage(state, |storage| {
                storage.reorder_item(&dragged, &target, pos)?;
                Ok(())
            });
        }
    }
    drag_id.set(None);
    drop_pos.set(None);
}

#[component]
fn TreeNode(
    state: Signal<AppState>,
    item: Item,
    depth: i32,
    children_map: HashMap<Option<String>, Vec<Item>>,
    mut drop_pos: Signal<Option<DropPos>>,
    mut drag_id: Signal<Option<String>>,
) -> Element {
    let active_id = state.read().active_item.as_ref().map(|i| i.id.clone());
    let is_active = active_id.as_deref() == Some(&item.id);
    let children = children_map.get(&Some(item.id.clone())).cloned().unwrap_or_default();
    let has_children = !children.is_empty();
    let is_container = can_accept_children(&item.item_type);
    let is_task = item.item_type == ItemType::Task;

    // Drag state
    let cur_drag = drag_id.read().clone();
    let is_foreign_drag = cur_drag.is_some() && cur_drag.as_deref() != Some(&item.id);
    let is_self_dragging = cur_drag.as_deref() == Some(&item.id);

    // Drop indicator state
    let dp = drop_pos.read().clone();
    let show_above = matches!(&dp, Some(DropPos::Above(x)) if x == &item.id);
    let show_into = matches!(&dp, Some(DropPos::Into(x)) if x == &item.id);
    let show_below = matches!(&dp, Some(DropPos::Below(x)) if x == &item.id);

    let icon = match item.item_type {
        ItemType::Folder => "F",
        ItemType::Document => "D",
        ItemType::Note => "N",
        ItemType::Project => "P",
        ItemType::Task => item.status.as_ref().map(|s| s.icon()).unwrap_or("○"),
    };

    let padding = format!("padding-left: {}px;", 12 + depth * 16);

    // Clones for event handlers
    let id_dragstart = item.id.clone();
    let id_click = item.id.clone();
    let id_delete = item.id.clone();
    let id_add_task = item.id.clone();
    let id_status = item.id.clone();
    let task_status = item.status.clone();

    // Clones for drop zone ondragover handlers
    let id_ov_above = item.id.clone();
    let id_ov_into = item.id.clone();
    let id_ov_below = item.id.clone();

    // Clones for drop zone ondrop handlers
    let id_dr_above = item.id.clone();
    let id_dr_into = item.id.clone();
    let id_dr_below = item.id.clone();

    // Build CSS classes
    let mut wrap_class = String::from("tree-node-wrap");
    if show_above { wrap_class.push_str(" drop-above"); }
    if show_into { wrap_class.push_str(" drop-into"); }
    if show_below { wrap_class.push_str(" drop-below"); }

    let mut node_class = String::from("tree-node");
    if is_active { node_class.push_str(" active"); }
    if is_self_dragging { node_class.push_str(" dragging"); }

    rsx! {
        div {
            class: "{wrap_class}",
            // Main visible node
            div {
                class: "{node_class}",
                style: "{padding}",
                draggable: "true",
                ondragstart: move |_| {
                    drag_id.set(Some(id_dragstart.clone()));
                    set_cross_drag(Some(&id_dragstart));
                },
                ondragend: move |_| {
                    drag_id.set(None);
                    drop_pos.set(None);
                    set_cross_drag(None);
                },
                onclick: move |_| {
                    flush_editor_pending(state);
                    let _ = with_storage(state, |storage| {
                        let loaded = storage.get_item(&id_click)?;
                        state.write().active_item = Some(loaded);
                        Ok(())
                    });
                },

                // Task status icon — clickable to cycle status
                if is_task {
                    span {
                        class: "node-icon status-click",
                        onclick: move |e| {
                            e.stop_propagation();
                            if let Some(ref st) = task_status {
                                let next = st.next();
                                update_item_field(state, &id_status, None, None, Some(next), None);
                            }
                        },
                        "{icon}"
                    }
                } else {
                    span { class: "node-icon", "{icon}" }
                }
                span { class: "node-title", "{item.title}" }
                // Quick add task button (hover-reveal, projects only)
                if is_container {
                    button {
                        class: "node-add-btn",
                        title: "Add task",
                        onclick: move |e| {
                            e.stop_propagation();
                            let parent = id_add_task.clone();
                            let _ = with_storage(state, |storage| {
                                let item = storage.create_item(CreateItemRequest {
                                    title: "New Task".to_string(),
                                    item_type: ItemType::Task,
                                    parent_id: Some(parent.clone()),
                                    content: None,
                                    status: Some(TaskStatus::Todo),
                                    priority: Some(TaskPriority::None),
                                })?;
                                state.write().active_item = Some(item);
                                Ok(())
                            });
                        },
                        "+"
                    }
                }
                // Delete button (hover-reveal)
                button {
                    class: "node-delete-btn",
                    onclick: move |e| {
                        e.stop_propagation();
                        let _ = with_storage(state, |storage| {
                            storage.delete_item(&id_delete)?;
                            Ok(())
                        });
                        // Clear active item if it was the deleted one
                        let active = state.read().active_item.as_ref().map(|i| i.id.clone());
                        if active.as_deref() == Some(&*id_delete) {
                            state.write().active_item = None;
                        }
                    },
                    "×"
                }
                if has_children {
                    span { class: "node-child-count", "{children.len()}" }
                }
            }

            // Drop zones — invisible overlays that appear only during a foreign drag.
            // For containers (Project/Folder): 3 zones (above / into / below).
            // For other items: 2 zones (above / below).
            if is_foreign_drag {
                if is_container {
                    div {
                        class: "drop-zone zone-top-third",
                        ondragover: move |e| { e.prevent_default(); drop_pos.set(Some(DropPos::Above(id_ov_above.clone()))); },
                        ondrop: move |e| {
                            e.stop_propagation();
                            do_drop(state, drag_id, drop_pos, &id_dr_above, ReorderPos::Before);
                        },
                    }
                    div {
                        class: "drop-zone zone-mid",
                        ondragover: move |e| { e.prevent_default(); drop_pos.set(Some(DropPos::Into(id_ov_into.clone()))); },
                        ondrop: move |e| {
                            e.stop_propagation();
                            do_drop(state, drag_id, drop_pos, &id_dr_into, ReorderPos::Into);
                        },
                    }
                    div {
                        class: "drop-zone zone-bot-third",
                        ondragover: move |e| { e.prevent_default(); drop_pos.set(Some(DropPos::Below(id_ov_below.clone()))); },
                        ondrop: move |e| {
                            e.stop_propagation();
                            do_drop(state, drag_id, drop_pos, &id_dr_below, ReorderPos::After);
                        },
                    }
                } else {
                    div {
                        class: "drop-zone zone-top-half",
                        ondragover: move |e| { e.prevent_default(); drop_pos.set(Some(DropPos::Above(id_ov_above.clone()))); },
                        ondrop: move |e| {
                            e.stop_propagation();
                            do_drop(state, drag_id, drop_pos, &id_dr_above, ReorderPos::Before);
                        },
                    }
                    div {
                        class: "drop-zone zone-bot-half",
                        ondragover: move |e| { e.prevent_default(); drop_pos.set(Some(DropPos::Below(id_ov_below.clone()))); },
                        ondrop: move |e| {
                            e.stop_propagation();
                            do_drop(state, drag_id, drop_pos, &id_dr_below, ReorderPos::After);
                        },
                    }
                }
            }
        }
        // Children
        for child in children.iter() {
            TreeNode { state, item: child.clone(), depth: depth + 1, children_map: children_map.clone(), drop_pos, drag_id }
        }
    }
}
