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
    // Wrap in setTimeout(0) so it runs AFTER Dioxus commits the DOM
    let js = format!(
        "setTimeout(function(){{{EDITOR_JS}\nwindow.__moteInitEditor(`{escaped}`);}},0);"
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

    // Detect item switch
    let current_id = item.id.clone();
    if *item_id.read() != current_id {
        content.set(item.content.clone().unwrap_or_default());
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
                            // Sync content from WYSIWYG before switching
                            dioxus::document::eval(
                                "if(window.__moteEditor){var b=document.getElementById('mote-content-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteEditor.getContent());b.dispatchEvent(new Event('input',{bubbles:true}));}}"
                            );
                            mode.set(EditorMode::Markdown);
                        } else {
                            // Switching back to RT — re-init editor with current markdown
                            editor_inited.set(false);
                            mode.set(EditorMode::Richtext);
                        }
                    },
                    if is_rt { "MD" } else { "RT" }
                }
            }

            // Editor body
            div { class: "editor-body",
                // WYSIWYG mount point — always in DOM, toggled via display
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
                            update_item_field(state, &id, None, Some(text), None, None);
                            clear_editor_pending();
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
                    }
                },
            }
        }
    }
}
