use dioxus::prelude::*;

use crate::models::*;
use crate::ui::app::{flush_editor_pending, update_item_field, with_storage, AppState};

#[component]
pub fn TaskView(state: Signal<AppState>, item: Item) -> Element {
    let mut title = use_signal(|| item.title.clone());
    let mut desc = use_signal(|| item.content.clone().unwrap_or_default());
    let mut new_task = use_signal(|| String::new());
    let mut filter = use_signal(|| TaskFilter::All);
    let mut item_id = use_signal(|| item.id.clone());
    // Drag state
    let mut dragging_id = use_signal(|| Option::<String>::None);
    let mut drop_target_id = use_signal(|| Option::<String>::None);

    let current_id = item.id.clone();
    if *item_id.read() != current_id {
        title.set(item.title.clone());
        desc.set(item.content.clone().unwrap_or_default());
        item_id.set(current_id);
    }

    let tree = state.read().tree.clone();
    let children: Vec<Item> = tree.iter()
        .filter(|i| i.parent_id.as_deref() == Some(&item.id) && !i.deleted)
        .cloned().collect();

    let current_filter = filter.read().clone();
    let filtered: Vec<Item> = children.iter()
        .filter(|c| current_filter.matches(c.status.as_ref()))
        .cloned().collect();

    let is_project = item.item_type == ItemType::Project;

    let (mut total, mut done, mut in_progress, mut todo_count) = (0, 0, 0, 0);
    for c in &children {
        if c.item_type == ItemType::Task {
            total += 1;
            match c.status.as_ref() {
                Some(TaskStatus::Done) => done += 1,
                Some(TaskStatus::InProgress) => in_progress += 1,
                Some(TaskStatus::Todo) => todo_count += 1,
                _ => {}
            }
        }
    }
    let pct = if total > 0 { (done as f64 / total as f64) * 100.0 } else { 0.0 };

    let parent_id_for_add = item.id.clone();
    let parent_id_for_add2 = item.id.clone();
    let parent_id_for_drop = item.id.clone();

    let filters = [TaskFilter::All, TaskFilter::Todo, TaskFilter::InProgress, TaskFilter::Done];

    rsx! {
        div { class: "task-view",
            div { class: "task-header",
                input {
                    class: "title-input",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                    onblur: move |_| {
                        let t = title.read().clone();
                        let id = item_id.read().clone();
                        update_item_field(state, &id, Some(t), None, None, None);
                    },
                }
                if item.item_type == ItemType::Task {
                    div { class: "task-meta",
                        button {
                            class: "meta-btn",
                            onclick: {
                                let ic = item.clone();
                                move |_| {
                                    let next = ic.status.as_ref().map(|s| s.next()).unwrap_or(TaskStatus::Todo);
                                    update_item_field(state, &ic.id, None, None, Some(next), None);
                                }
                            },
                            {format!("{} {}", item.status.as_ref().map(|s| s.icon()).unwrap_or("○"), item.status.as_ref().map(|s| s.label()).unwrap_or("Todo"))}
                        }
                        button {
                            class: "meta-btn",
                            onclick: {
                                let ic = item.clone();
                                move |_| {
                                    let next = ic.priority.as_ref().map(|p| p.next()).unwrap_or(TaskPriority::None);
                                    update_item_field(state, &ic.id, None, None, None, Some(next));
                                }
                            },
                            {format!("{}", item.priority.as_ref().map(|p| p.label()).unwrap_or("-"))}
                        }
                    }
                }
            }

            if is_project {
                div { class: "project-stats",
                    span { "{total} tasks" }
                    span { style: "color: #4ade80;", "{done} done" }
                    span { style: "color: #fbbf24;", "{in_progress} in progress" }
                    span { style: "color: rgba(255,255,255,0.3);", "{todo_count} todo" }
                }
                if total > 0 {
                    div { class: "progress-bar",
                        div { class: "progress-fill", style: "width: {pct}%;" }
                    }
                }
            }

            if item.item_type == ItemType::Task {
                textarea {
                    class: "desc-textarea",
                    placeholder: "Add description (markdown)...",
                    value: "{desc}",
                    oninput: move |e| desc.set(e.value()),
                    onblur: move |_| {
                        let d = desc.read().clone();
                        let id = item_id.read().clone();
                        update_item_field(state, &id, None, Some(d), None, None);
                    },
                }
            }

            div {
                div { class: "subtasks-header",
                    h3 { "Sub-tasks" }
                    div { class: "filter-btns",
                        for f in filters.iter() {
                            button {
                                class: if current_filter == *f { "active" } else { "" },
                                onclick: {
                                    let fv = f.clone();
                                    move |_| filter.set(fv.clone())
                                },
                                "{f.label()}"
                            }
                        }
                    }
                }

                div { class: "add-task",
                    input {
                        r#type: "text",
                        placeholder: "Add a task...",
                        value: "{new_task}",
                        oninput: move |e| new_task.set(e.value()),
                        onkeypress: move |e| {
                            if e.key() == Key::Enter {
                                add_task_from_input(state, &parent_id_for_add, &mut new_task);
                            }
                        },
                    }
                    button {
                        class: "add-task-btn",
                        onclick: move |_| {
                            add_task_from_input(state, &parent_id_for_add2, &mut new_task);
                        },
                        "Add"
                    }
                }

                // Draggable task list
                div {
                    class: "task-list",
                    // Drop zone at the very end (for dropping as last item)
                    ondragover: move |e| { e.prevent_default(); },
                    ondrop: {
                        let pid = parent_id_for_drop.clone();
                        let flen = filtered.len() as i32;
                        move |_| {
                            let drag_id = dragging_id.read().clone();
                            if let Some(drag_id) = drag_id {
                                let new_order = flen * 10;
                                let _ = with_storage(state, |storage| {
                                    storage.move_item(&drag_id, Some(&pid), new_order)?;
                                    Ok(())
                                });
                            }
                            dragging_id.set(None);
                            drop_target_id.set(None);
                        }
                    },

                    for (idx, task) in filtered.iter().enumerate() {
                        DraggableTaskItem {
                            state,
                            task: task.clone(),
                            idx: idx as i32,
                            dragging_id,
                            drop_target_id,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DraggableTaskItem(
    state: Signal<AppState>,
    task: Item,
    idx: i32,
    mut dragging_id: Signal<Option<String>>,
    mut drop_target_id: Signal<Option<String>>,
) -> Element {
    let is_done = task.status.as_ref() == Some(&TaskStatus::Done);
    let status_icon = task.status.as_ref().map(|s| s.icon()).unwrap_or("○");
    let status_label = task.status.as_ref().map(|s| s.label()).unwrap_or("Todo");
    let priority_label = task.priority.as_ref().map(|p| p.label()).unwrap_or("-");
    let priority_color = task.priority.as_ref().map(|p| p.color()).unwrap_or("#5a6577");

    let tid = task.id.clone();
    let tid_toggle = task.id.clone();
    let tid_select = task.id.clone();
    let tid_cycle = task.id.clone();
    let tid_delete = task.id.clone();
    let tid_drag = task.id.clone();
    let tid_drop = task.id.clone();
    let task_for_cycle = task.clone();

    let is_dragging = dragging_id.read().as_deref() == Some(&tid);
    let is_drop_target = drop_target_id.read().as_deref() == Some(&tid);

    let class = {
        let mut c = String::from("task-item");
        if is_done { c.push_str(" done"); }
        if is_dragging { c.push_str(" dragging"); }
        if is_drop_target { c.push_str(" drop-target"); }
        c
    };

    let parent_id = task.parent_id.clone();

    rsx! {
        div {
            class: "{class}",
            draggable: "true",

            ondragstart: move |_| {
                dragging_id.set(Some(tid_drag.clone()));
            },
            ondragend: move |_| {
                dragging_id.set(None);
                drop_target_id.set(None);
            },
            ondragover: move |e| {
                e.prevent_default();
                drop_target_id.set(Some(tid_drop.clone()));
            },
            ondragleave: move |_| {
                drop_target_id.set(None);
            },
            ondrop: {
                let target_id = tid.clone();
                let target_idx = idx;
                move |e| {
                    e.stop_propagation();
                    if let Some(drag_id) = dragging_id.read().clone() {
                        if drag_id != target_id {
                            // Move dragged item to the position of the drop target
                            let new_order = target_idx * 10;
                            let pid = parent_id.clone();
                            let _ = with_storage(state, |storage| {
                                storage.move_item(&drag_id, pid.as_deref(), new_order)?;
                                Ok(())
                            });
                        }
                    }
                    dragging_id.set(None);
                    drop_target_id.set(None);
                }
            },

            // Drag handle
            span { class: "drag-handle", "⠿" }

            button {
                class: "status-btn",
                onclick: move |_| {
                    let next = if is_done { TaskStatus::Todo } else { TaskStatus::Done };
                    update_item_field(state, &tid_toggle, None, None, Some(next), None);
                },
                "{status_icon}"
            }
            span {
                class: if is_done { "task-title done-text" } else { "task-title" },
                onclick: move |_| {
                    flush_editor_pending(state);
                    let _ = with_storage(state, |storage| {
                        let loaded = storage.get_item(&tid_select)?;
                        state.write().active_item = Some(loaded);
                        Ok(())
                    });
                },
                "{task.title}"
            }
            if priority_label != "-" {
                span { class: "priority-badge", style: "color: {priority_color};", "{priority_label}" }
            }
            button {
                class: "status-label",
                onclick: move |_| {
                    let next = task_for_cycle.status.as_ref().map(|s| s.next()).unwrap_or(TaskStatus::Todo);
                    update_item_field(state, &tid_cycle, None, None, Some(next), None);
                },
                "{status_label}"
            }
            button {
                class: "delete-btn",
                onclick: move |_| {
                    let id = tid_delete.clone();
                    let _ = with_storage(state, |storage| {
                        storage.delete_item(&id)?;
                        let mut st = state.write();
                        if st.active_item.as_ref().is_some_and(|i| i.id == id) {
                            st.active_item = None;
                        }
                        Ok(())
                    });
                },
                "x"
            }
        }
    }
}

fn add_task_from_input(state: Signal<AppState>, parent_id: &str, new_task: &mut Signal<String>) {
    let t = new_task.read().clone();
    if t.trim().is_empty() {
        return;
    }
    let pid = parent_id.to_string();
    let _ = with_storage(state, |storage| {
        storage.create_item(CreateItemRequest {
            title: t.trim().to_string(),
            item_type: ItemType::Task,
            parent_id: Some(pid),
            content: None,
            status: Some(TaskStatus::Todo),
            priority: Some(TaskPriority::None),
        })?;
        Ok(())
    });
    new_task.set(String::new());
}
