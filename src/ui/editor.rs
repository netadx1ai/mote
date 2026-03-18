use dioxus::prelude::*;

use crate::models::*;
use crate::ui::app::{clear_editor_pending, set_editor_pending, update_item_field, AppState};
use crate::ui::markdown::render_markdown;

const EDITOR_JS: &str = include_str!("editor_init.js");

#[derive(Clone, PartialEq)]
enum EditorMode {
    Richtext,
    Markdown,
}

fn eval_init_editor(html: &str) {
    let escaped = html.replace('\\', "\\\\").replace('`', "\\`");
    let js = format!(
        "setTimeout(function(){{{EDITOR_JS}\nwindow.__moteInitEditor(`{escaped}`);}},0);"
    );
    dioxus::document::eval(&js);
}

/// Save content to storage directly (called by auto-save timer)
fn save_now(state: Signal<AppState>, id: &str, content: &str) {
    let id = id.to_string();
    let content = content.to_string();
    update_item_field(state, &id, None, Some(content), None, None);
    clear_editor_pending();
}

/// Pull latest HTML from WYSIWYG editor into the content signal
fn sync_rt_to_content(content: &mut Signal<String>, item_id: &Signal<String>) {
    dioxus::document::eval(
        "if(window.__moteEditor){var b=document.getElementById('mote-content-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteEditor.getContent());b.dispatchEvent(new Event('input',{bubbles:true}));}}"
    );
}

#[component]
pub fn Editor(state: Signal<AppState>, item: Item) -> Element {
    let mut content = use_signal(|| item.content.clone().unwrap_or_default());
    let mut title = use_signal(|| item.title.clone());
    let mut item_id = use_signal(|| item.id.clone());
    let mut mode = use_signal(|| EditorMode::Richtext);
    let mut editor_inited = use_signal(|| false);
    let mut last_saved = use_signal(|| item.content.clone().unwrap_or_default());

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

        content.set(item.content.clone().unwrap_or_default());
        last_saved.set(item.content.clone().unwrap_or_default());
        title.set(item.title.clone());
        item_id.set(current_id.clone());
        editor_inited.set(false);
        clear_editor_pending();
    }

    let current_mode = mode.read().clone();
    let is_rt = current_mode == EditorMode::Richtext;

    // Init RT editor when in RT mode and not yet initialized for this item
    if is_rt && !*editor_inited.read() {
        let html = render_markdown(&content.read());
        eval_init_editor(&html);
        editor_inited.set(true);
    }

    // Auto-save: use JS setInterval to trigger save every 3 seconds
    // The JS calls syncContent which updates the bridge, then we save from Rust
    use_effect(move || {
        dioxus::document::eval(
            "if(!window.__moteAutoSave){window.__moteAutoSave=setInterval(function(){if(window.__moteEditor){var b=document.getElementById('mote-content-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteEditor.getContent());b.dispatchEvent(new Event('input',{bubbles:true}));}}},3000);}"
        );
    });

    let word_count = content.read().split_whitespace().count();
    let line_count = content.read().lines().count();

    rsx! {
        div { class: "editor-container",
            // Title + mode toggle
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
                button {
                    class: "mode-switch",
                    title: if is_rt { "Switch to Markdown" } else { "Switch to Rich Text" },
                    onclick: move |_| {
                        if *mode.read() == EditorMode::Richtext {
                            sync_rt_to_content(&mut content, &item_id);
                            mode.set(EditorMode::Markdown);
                        } else {
                            editor_inited.set(false);
                            mode.set(EditorMode::Richtext);
                        }
                    },
                    if is_rt { "MD" } else { "RT" }
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
                // Raw markdown textarea
                if !is_rt {
                    textarea {
                        class: "editor-textarea",
                        value: "{content}",
                        placeholder: "Write markdown...",
                        oninput: move |e| {
                            content.set(e.value());
                            set_editor_pending(&item_id.read(), &content.read());
                        },
                        onblur: move |_| {
                            let text = content.read().clone();
                            let id = item_id.read().clone();
                            save_now(state, &id, &text);
                            last_saved.set(text);
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
                        let prev = content.read().clone();
                        content.set(html.clone());
                        set_editor_pending(&item_id.read(), &html);
                        // Auto-save if content actually changed
                        if html != prev {
                            let id = item_id.read().clone();
                            save_now(state, &id, &html);
                            last_saved.set(html);
                        }
                    }
                },
            }
        }
    }
}
