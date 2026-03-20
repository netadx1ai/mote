use dioxus::prelude::*;

use crate::models::*;
use crate::ui::app::{clear_editor_pending, set_editor_pending, update_item_field, AppState};
use crate::ui::markdown::render_markdown;

fn save_now(state: Signal<AppState>, id: &str, content: &str) {
    let id = id.to_string();
    let content = content.to_string();
    update_item_field(state, &id, None, Some(content), None, None);
    clear_editor_pending();
}

#[component]
pub fn Editor(state: Signal<AppState>, item: Item) -> Element {
    let mut content = use_signal(|| item.content.clone().unwrap_or_default());
    let mut title = use_signal(|| item.title.clone());
    let mut item_id = use_signal(|| item.id.clone());
    let mut last_saved = use_signal(|| item.content.clone().unwrap_or_default());
    // Holds (prev_id, prev_content) that need saving after a render cycle
    let mut pending_save: Signal<Option<(String, String)>> = use_signal(|| None);

    // Detect item switch in render body — only mutate local signals, not AppState
    let current_id = item.id.clone();
    if *item_id.read() != current_id {
        let prev_id = item_id.read().clone();
        let prev_content = content.read().clone();
        let prev_saved = last_saved.read().clone();
        if prev_content != prev_saved && !prev_id.is_empty() {
            pending_save.set(Some((prev_id, prev_content)));
        }
        content.set(item.content.clone().unwrap_or_default());
        last_saved.set(item.content.clone().unwrap_or_default());
        title.set(item.title.clone());
        item_id.set(current_id.clone());
        clear_editor_pending();
    }

    // Deferred AppState mutation — runs after render, safe from render body
    use_effect(move || {
        if let Some((id, text)) = pending_save.read().clone() {
            save_now(state, &id, &text);
            pending_save.set(None);
        }
    });

    let word_count = content.read().split_whitespace().count();
    let line_count = content.read().lines().count();
    let preview_html = render_markdown(&content.read());

    rsx! {
        div { class: "editor-container editor-area",
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
            }

            div { class: "editor-toolbar",
                button {
                    class: "toolbar-btn",
                    title: "Bold",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        c.push_str("**bold**");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "B"
                }
                button {
                    class: "toolbar-btn",
                    title: "Italic",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        c.push_str("*italic*");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "I"
                }
                button {
                    class: "toolbar-btn",
                    title: "Code",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        c.push_str("`code`");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "`"
                }
                button {
                    class: "toolbar-btn",
                    title: "Link",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        c.push_str("[text](url)");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "[]"
                }
                button {
                    class: "toolbar-btn",
                    title: "Heading 1",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        if !c.is_empty() && !c.ends_with('\n') {
                            c.push('\n');
                        }
                        c.push_str("# ");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "H1"
                }
                button {
                    class: "toolbar-btn",
                    title: "Heading 2",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        if !c.is_empty() && !c.ends_with('\n') {
                            c.push('\n');
                        }
                        c.push_str("## ");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "H2"
                }
                button {
                    class: "toolbar-btn",
                    title: "Heading 3",
                    onclick: move |_| {
                        let mut c = content.read().clone();
                        if !c.is_empty() && !c.ends_with('\n') {
                            c.push('\n');
                        }
                        c.push_str("### ");
                        content.set(c.clone());
                        set_editor_pending(&item_id.read(), &c);
                    },
                    "H3"
                }
            }

            div { class: "editor-split",
                textarea {
                    class: "editor-textarea",
                    placeholder: "Write markdown...",
                    value: "{content}",
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
                    onkeydown: move |e| {
                        let mods = e.modifiers();
                        if (mods.ctrl() || mods.meta()) && e.key() == Key::Character("s".to_string()) {
                            let text = content.read().clone();
                            let id = item_id.read().clone();
                            save_now(state, &id, &text);
                            last_saved.set(text);
                        }
                    },
                }
                div {
                    class: "markdown-preview",
                    dangerous_inner_html: "{preview_html}",
                }
            }
        }
    }
}
