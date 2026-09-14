use dioxus::prelude::*;

use crate::models::*;
use crate::ui::app::{clear_editor_pending, save_editor_content, set_editor_pending, update_item_field, AppState};
use crate::ui::markdown::render_markdown;

const EDITOR_JS: &str = include_str!("editor_init.js");

#[derive(Clone, Copy, PartialEq)]
enum EditorMode {
    Richtext,
    Markdown,
    Monaco,
}

fn eval_init_editor(html: &str) {
    let escaped = html.replace('\\', "\\\\").replace('`', "\\`");
    let js = format!(
        "setTimeout(function(){{{EDITOR_JS}\nwindow.__moteInitEditor(`{escaped}`);}},0);"
    );
    dioxus::document::eval(&js);
}

/// Save content to storage directly
fn save_now(state: Signal<AppState>, id: &str, content: &str) {
    save_editor_content(state, id, content.to_string());
    clear_editor_pending();
}

/// Pull latest HTML from WYSIWYG editor into the content signal
fn sync_rt_to_content() {
    dioxus::document::eval(
        "if(window.__moteEditor){var b=document.getElementById('mote-content-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteEditor.getContent());b.dispatchEvent(new Event('input',{bubbles:true}));}}"
    );
}

/// Initialize Monaco editor for document/notes editor
fn init_monaco_editor(content: &str) {
    let escaped = content.replace('\\', "\\\\").replace('`', "\\`").replace("${", "\\${");
    let js = format!(
        "setTimeout(function(){{var p={{content:`{escaped}`,language:'markdown',tabId:'doc-editor'}};window.__moteMonacoMount(p.content,p.language,p.tabId);}},100);"
    );
    dioxus::document::eval(&js);
}

/// Sync Monaco editor content to bridge
fn sync_monaco_to_bridge() {
    dioxus::document::eval(
        "if(window.__moteMonaco&&window.__moteMonaco.editor){var b=document.getElementById('monaco-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteMonaco.editor.getValue());b.dispatchEvent(new Event('input',{bubbles:true}));}}"
    );
}

/// Update Monaco editor content (for tab switches)
fn update_monaco_content(content: &str) {
    let escaped = content.replace('\\', "\\\\").replace('`', "\\`").replace("${", "\\${");
    let js = format!(
        "setTimeout(function(){{window.__moteMonacoSetContent(`{escaped}`,'markdown','doc-editor');}},0);"
    );
    dioxus::document::eval(&js);
}

#[component]
pub fn Editor(state: Signal<AppState>, item: Item) -> Element {
    let mut content = use_signal(|| item.content.clone().unwrap_or_default());
    let mut title = use_signal(|| item.title.clone());
    let mut item_id = use_signal(|| item.id.clone());
    let mut mode = use_signal(|| EditorMode::Richtext);
    let mut editor_inited = use_signal(|| false);
    let mut monaco_inited = use_signal(|| false);
    let mut monaco_view = use_signal(|| true); // true = Monaco, false = Text view
    let mut last_saved = use_signal(|| item.content.clone().unwrap_or_default());
    let mut save_status = use_signal(|| "Saved"); // "Saved", "Saving...", "Unsaved"
    let mut edit_tick = use_signal(|| 0u64);

    // Detect item switch — save previous item first, then load new
    let current_id = item.id.clone();
    if *item_id.read() != current_id {
        // Save previous item if content changed
        let prev_id = item_id.read().clone();
        let prev_content = content.read().clone();
        let prev_saved = last_saved.read().clone();
        if prev_content != prev_saved && !prev_id.is_empty() {
            save_now(state, &prev_id, &prev_content);
        }

        let new_content = item.content.clone().unwrap_or_default();
        content.set(new_content.clone());
        last_saved.set(new_content);
        title.set(item.title.clone());
        item_id.set(current_id.clone());
        editor_inited.set(false);
        monaco_inited.set(false);
        monaco_view.set(true);
        save_status.set("Saved");
        clear_editor_pending();
    }

    let current_mode = mode.read().clone();
    let is_rt = current_mode == EditorMode::Richtext;
    let is_monaco = current_mode == EditorMode::Monaco;
    let show_monaco = is_monaco && *monaco_view.read();
    let show_textarea = !is_rt && (!is_monaco || !*monaco_view.read());

    // Init Monaco editor when in Monaco mode and not yet initialized
    if is_monaco && !*monaco_inited.read() {
        monaco_inited.set(true);
        init_monaco_editor(&content.read());
    }

    // Init RT editor when in RT mode and not yet initialized for this item
    if is_rt && !*editor_inited.read() {
        let html = render_markdown(&content.read());
        eval_init_editor(&html);
        editor_inited.set(true);
    }

    // Debounced auto-save engine: runs 1.2s after last typing activity
    use_future(move || {
        let tick = *edit_tick.read();
        async move {
            if tick == 0 {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
            if *edit_tick.peek() == tick {
                let id = item_id.read().clone();
                let text = content.read().clone();
                if !id.is_empty() && text != *last_saved.peek() {
                    save_status.set("Saving...");
                    save_now(state, &id, &text);
                    last_saved.set(text);
                    save_status.set("Saved");
                }
            }
        }
    });

    // Periodic safety sync from JS editor
    use_effect(move || {
        dioxus::document::eval(
            "if(!window.__moteAutoSave){window.__moteAutoSave=setInterval(function(){if(window.__moteEditor){var b=document.getElementById('mote-content-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteEditor.getContent());b.dispatchEvent(new Event('input',{bubbles:true}));}}},3000);}"
        );
    });

    let word_count = content.read().split_whitespace().count();
    let line_count = content.read().lines().count();

    rsx! {
        div { class: "editor-container",
            // Title + mode toggle + save status
            div { class: "editor-header",
                input {
                    class: "title-input",
                    placeholder: "Untitled",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                    onblur: move |_| {
                        let t = title.read().clone();
                        let id = item_id.read().clone();
                        update_item_field(state, &id, Some(t), None, None, None);
                    },
                }
                span { class: "editor-stats", "{word_count}w · {line_count}L" }

                // Save indicator & manual save button
                span {
                    class: match *save_status.read() {
                        "Saved" => "save-indicator saved",
                        "Saving..." => "save-indicator saving",
                        _ => "save-indicator unsaved",
                    },
                    title: "Auto-saves automatically, or press Cmd+S",
                    match *save_status.read() {
                        "Saved" => "✓ Saved",
                        "Saving..." => "Saving...",
                        _ => "● Unsaved",
                    }
                }

                button {
                    class: "btn-save",
                    title: "Save immediately (Cmd+S)",
                    onclick: move |_| {
                        let id = item_id.read().clone();
                        let text = content.read().clone();
                        if !id.is_empty() {
                            save_status.set("Saving...");
                            save_now(state, &id, &text);
                            last_saved.set(text);
                            save_status.set("Saved");
                        }
                    },
                    "Save"
                }

                if is_monaco {
                    button {
                        class: "mode-switch",
                        title: "Swap between Monaco and Text view",
                        onclick: move |_| {
                            let is_currently_monaco = *monaco_view.read();
                            if is_currently_monaco {
                                sync_monaco_to_bridge();
                            } else {
                                let current_content = content.read().clone();
                                update_monaco_content(&current_content);
                            }
                            monaco_view.set(!is_currently_monaco);
                        },
                        "⇄"
                    }
                }
                button {
                    class: "mode-switch",
                    title: if is_rt { "Switch to Markdown" } else if is_monaco { "Switch to Rich Text" } else { "Switch to Monaco" },
                    onclick: move |_| {
                        let current = *mode.read();
                        match current {
                            EditorMode::Richtext => {
                                sync_rt_to_content();
                                mode.set(EditorMode::Markdown);
                            }
                            EditorMode::Markdown => {
                                mode.set(EditorMode::Monaco);
                            }
                            EditorMode::Monaco => {
                                sync_monaco_to_bridge();
                                editor_inited.set(false);
                                monaco_inited.set(false);
                                mode.set(EditorMode::Richtext);
                            }
                        }
                    },
                    if is_rt { "MD" } else if is_monaco { "RT" } else { "MX" }
                }
            }

            // Editor body
            div { class: "editor-body",
                // WYSIWYG mount point
                div {
                    id: "mote-editor-wrap",
                    class: "mote-editor-wrap",
                    style: if is_rt { "" } else { "display:none;" },
                }
                // Monaco editor container
                div {
                    id: "monaco-container",
                    class: "editor-monaco",
                    style: if show_monaco { "" } else { "display:none;" },
                }
                // Raw markdown textarea
                if show_textarea {
                    textarea {
                        class: "editor-textarea",
                        value: "{content}",
                        placeholder: "Write markdown...",
                        oninput: move |e| {
                            let val = e.value();
                            content.set(val.clone());
                            set_editor_pending(&item_id.read(), &val);
                            save_status.set("Unsaved");
                            *edit_tick.write() += 1;
                        },
                        onblur: move |_| {
                            let text = content.read().clone();
                            let id = item_id.read().clone();
                            save_status.set("Saving...");
                            save_now(state, &id, &text);
                            last_saved.set(text);
                            save_status.set("Saved");
                        },
                    }
                }
            }

            // Hidden bridge for WYSIWYG content sync
            input {
                id: "mote-content-bridge",
                r#type: "hidden",
                value: "",
                oninput: move |e| {
                    let html = e.value();
                    if !html.is_empty() {
                        content.set(html.clone());
                        set_editor_pending(&item_id.read(), &html);
                        save_status.set("Unsaved");
                        *edit_tick.write() += 1;
                    }
                },
            }

            // Hidden bridge for Monaco content sync
            input {
                id: "monaco-bridge",
                r#type: "hidden",
                value: "",
                oninput: move |e| {
                    let val = e.value();
                    if !val.is_empty() {
                        content.set(val.clone());
                        set_editor_pending(&item_id.read(), &val);
                        save_status.set("Unsaved");
                        *edit_tick.write() += 1;
                    }
                },
            }

            // Hidden save bridge for keyboard shortcut Cmd+S & Monaco save
            button {
                id: "mote-save-bridge",
                style: "display: none;",
                onclick: move |_| {
                    let id = item_id.read().clone();
                    let text = content.read().clone();
                    if !id.is_empty() {
                        save_status.set("Saving...");
                        save_now(state, &id, &text);
                        last_saved.set(text);
                        save_status.set("Saved");
                    }
                },
            }
        }
    }
}
