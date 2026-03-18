use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::models::*;
use crate::storage::backup;
use crate::storage::Storage;
use crate::ui::app::{open_workspace, AppState};

#[component]
pub fn Settings(state: Signal<AppState>) -> Element {
    let mut status_msg = use_signal(|| Option::<(String, bool)>::None);
    let mut backup_path_input = use_signal(|| String::new());
    let mut restore_path_input = use_signal(|| String::new());
    let mut export_path_input = use_signal(|| String::new());
    let mut import_path_input = use_signal(|| String::new());
    let mut new_workspace_input = use_signal(|| String::new());

    let st = state.read();
    let workspace_path_str = st.workspace_path.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "Not set".to_string());

    // Single-pass stats
    let (mut total, mut docs, mut tasks, mut notes, mut projects, mut done) = (0, 0, 0, 0, 0, 0);
    for i in &st.tree {
        if i.deleted { continue; }
        total += 1;
        match i.item_type {
            ItemType::Document => docs += 1,
            ItemType::Task => { tasks += 1; if i.status.as_ref() == Some(&TaskStatus::Done) { done += 1; } }
            ItemType::Note => notes += 1,
            ItemType::Project => projects += 1,
            ItemType::Folder => {}
        }
    }
    drop(st);

    rsx! {
        div { class: "settings",
            h2 { "Settings" }

            // Workspace
            div { class: "settings-section",
                h3 { "Workspace" }
                div { class: "settings-row",
                    label { "Path:" }
                    span { class: "value", "{workspace_path_str}" }
                }
                div { class: "settings-row",
                    input {
                        class: "workspace-input", style: "flex: 1;",
                        placeholder: "New workspace path...",
                        value: "{new_workspace_input}",
                        oninput: move |e| new_workspace_input.set(e.value()),
                    }
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            let p = new_workspace_input.read().clone();
                            if p.is_empty() { set_msg(&mut status_msg, "Enter a path first", false); return; }
                            match open_workspace(state, PathBuf::from(&p)) {
                                Ok(()) => { new_workspace_input.set(String::new()); set_msg(&mut status_msg, "Workspace changed", true); }
                                Err(e) => set_msg(&mut status_msg, &format!("Failed: {e}"), false),
                            }
                        },
                        "Switch Workspace"
                    }
                }
            }

            // Stats
            div { class: "settings-section",
                h3 { "Statistics" }
                div { class: "stats-grid",
                    StatCard { value: total, label: "Total items" }
                    StatCard { value: docs, label: "Documents" }
                    StatCard { value: tasks, label: "Tasks" }
                    StatCard { value: notes, label: "Notes" }
                    StatCard { value: projects, label: "Projects" }
                    StatCard { value: done, label: "Tasks done" }
                }
            }

            // Backup
            div { class: "settings-section",
                h3 { "Backup" }
                p { style: "font-size: 12px; color: #5a6577; margin-bottom: 10px;",
                    "Create a zip archive of your entire workspace."
                }
                div { class: "settings-row",
                    input {
                        class: "workspace-input", style: "flex: 1;",
                        placeholder: "Output directory (e.g. ~/backups)",
                        value: "{backup_path_input}",
                        oninput: move |e| backup_path_input.set(e.value()),
                    }
                    button {
                        class: "btn-success",
                        onclick: move |_| {
                            let out_dir = backup_path_input.read().clone();
                            let wp = state.read().workspace_path.clone();
                            if let Some(wp) = wp {
                                let out = if out_dir.is_empty() { wp.parent().unwrap_or(&wp).to_path_buf() } else { PathBuf::from(&out_dir) };
                                if !out.exists() { let _ = std::fs::create_dir_all(&out); }
                                match backup::create_backup(&crate::storage::data_dir(&wp), &out) {
                                    Ok(p) => set_msg(&mut status_msg, &format!("Backup: {}", p.display()), true),
                                    Err(e) => set_msg(&mut status_msg, &format!("Backup failed: {e}"), false),
                                }
                            } else { set_msg(&mut status_msg, "No workspace", false); }
                        },
                        "Create Backup"
                    }
                }
            }

            // Restore
            div { class: "settings-section",
                h3 { "Restore" }
                p { style: "font-size: 12px; color: #5a6577; margin-bottom: 10px;",
                    "Restore from zip backup. Replaces current data."
                }
                div { class: "settings-row",
                    input {
                        class: "workspace-input", style: "flex: 1;",
                        placeholder: "Path to .zip file",
                        value: "{restore_path_input}",
                        oninput: move |e| restore_path_input.set(e.value()),
                    }
                    button {
                        class: "btn-danger",
                        onclick: move |_| {
                            let zip = restore_path_input.read().clone();
                            if zip.is_empty() { set_msg(&mut status_msg, "Enter backup path", false); return; }
                            let wp = state.read().workspace_path.clone();
                            if let Some(wp) = wp {
                                // Release DB locks
                                state.write().storage = None;
                                let result = backup::restore_backup(&PathBuf::from(&zip), &crate::storage::data_dir(&wp));
                                // Re-open storage
                                reopen_storage(state, &wp);
                                match result {
                                    Ok(()) => set_msg(&mut status_msg, "Restore complete", true),
                                    Err(e) => set_msg(&mut status_msg, &format!("Restore failed: {e}"), false),
                                }
                            } else { set_msg(&mut status_msg, "No workspace", false); }
                        },
                        "Restore"
                    }
                }
            }

            // Export/Import JSON
            div { class: "settings-section",
                h3 { "Export / Import Data" }
                p { style: "font-size: 12px; color: #5a6577; margin-bottom: 10px;",
                    "Portable JSON export/import."
                }
                div { class: "settings-row",
                    input {
                        class: "workspace-input", style: "flex: 1;",
                        placeholder: "Export path (e.g. ~/export.json)",
                        value: "{export_path_input}",
                        oninput: move |e| export_path_input.set(e.value()),
                    }
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            let out = export_path_input.read().clone();
                            if out.is_empty() { set_msg(&mut status_msg, "Enter export path", false); return; }
                            let st = state.read();
                            if let Some(ref storage) = st.storage {
                                let storage = storage.clone();
                                drop(st);
                                match backup::export_json(&storage) {
                                    Ok(json) => match std::fs::write(&out, &json) {
                                        Ok(()) => set_msg(&mut status_msg, &format!("Exported to {out}"), true),
                                        Err(e) => set_msg(&mut status_msg, &format!("Write failed: {e}"), false),
                                    },
                                    Err(e) => set_msg(&mut status_msg, &format!("Export failed: {e}"), false),
                                }
                            } else { set_msg(&mut status_msg, "No workspace", false); }
                        },
                        "Export JSON"
                    }
                }
                div { class: "settings-row",
                    input {
                        class: "workspace-input", style: "flex: 1;",
                        placeholder: "Import path (e.g. ~/export.json)",
                        value: "{import_path_input}",
                        oninput: move |e| import_path_input.set(e.value()),
                    }
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            let inp = import_path_input.read().clone();
                            if inp.is_empty() { set_msg(&mut status_msg, "Enter import path", false); return; }
                            match std::fs::read_to_string(&inp) {
                                Ok(json) => {
                                    let st = state.read();
                                    if let Some(ref storage) = st.storage {
                                        let storage = storage.clone();
                                        drop(st);
                                        match backup::import_json(&storage, &json) {
                                            Ok(n) => {
                                                let tree = storage.get_tree().unwrap_or_default();
                                                state.write().tree = tree;
                                                set_msg(&mut status_msg, &format!("Imported {n} items"), true);
                                            }
                                            Err(e) => set_msg(&mut status_msg, &format!("Import failed: {e}"), false),
                                        }
                                    } else { set_msg(&mut status_msg, "No workspace", false); }
                                }
                                Err(e) => set_msg(&mut status_msg, &format!("Read failed: {e}"), false),
                            }
                        },
                        "Import JSON"
                    }
                }
            }

            // Status
            {
                let msg = status_msg.read().clone();
                if let Some((text, ok)) = msg {
                    rsx! {
                        div {
                            class: if ok { "status-msg success" } else { "status-msg error" },
                            "{text}"
                            button {
                                style: "float: right; background: none; border: none; color: inherit; cursor: pointer; font-family: inherit;",
                                onclick: move |_| status_msg.set(None),
                                "x"
                            }
                        }
                    }
                } else { rsx! { {} } }
            }
        }
    }
}

#[component]
fn StatCard(value: usize, label: &'static str) -> Element {
    rsx! {
        div { class: "stat-card",
            div { class: "stat-value", "{value}" }
            div { class: "stat-label", "{label}" }
        }
    }
}

fn set_msg(sig: &mut Signal<Option<(String, bool)>>, text: &str, ok: bool) {
    sig.set(Some((text.to_string(), ok)));
}

fn reopen_storage(mut state: Signal<AppState>, wp: &PathBuf) {
    if let Ok(storage) = Storage::new(wp) {
        let tree = storage.get_tree().unwrap_or_default();
        let mut st = state.write();
        st.storage = Some(Arc::new(storage));
        st.tree = tree;
        st.active_item = None;
    }
}
