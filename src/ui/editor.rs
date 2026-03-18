use dioxus::prelude::*;

use crate::models::*;
use crate::ui::app::{clear_editor_pending, set_editor_pending, update_item_field, AppState};
use crate::ui::markdown::render_markdown;

#[derive(Clone, PartialEq)]
enum Mode {
    Split,
    Edit,
    Preview,
}

#[derive(Clone)]
struct SlashCommand {
    label: &'static str,
    icon: &'static str,
    snippet: &'static str,
    description: &'static str,
}

const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand { label: "Heading 1", icon: "H1", snippet: "# ", description: "Large heading" },
    SlashCommand { label: "Heading 2", icon: "H2", snippet: "## ", description: "Medium heading" },
    SlashCommand { label: "Heading 3", icon: "H3", snippet: "### ", description: "Small heading" },
    SlashCommand { label: "Bullet List", icon: "•", snippet: "- ", description: "Unordered list" },
    SlashCommand { label: "Numbered List", icon: "1.", snippet: "1. ", description: "Ordered list" },
    SlashCommand { label: "Task List", icon: "☐", snippet: "- [ ] ", description: "Checkbox item" },
    SlashCommand { label: "Quote", icon: "❝", snippet: "> ", description: "Block quote" },
    SlashCommand { label: "Code Block", icon: "<>", snippet: "```\n\n```", description: "Fenced code" },
    SlashCommand { label: "Divider", icon: "—", snippet: "\n---\n", description: "Horizontal rule" },
    SlashCommand { label: "Table", icon: "▦", snippet: "| Col 1 | Col 2 | Col 3 |\n|--------|--------|--------|\n| | | |\n", description: "Markdown table" },
    SlashCommand { label: "Bold", icon: "B", snippet: "****", description: "Bold text" },
    SlashCommand { label: "Italic", icon: "I", snippet: "__", description: "Italic text" },
    SlashCommand { label: "Link", icon: "🔗", snippet: "[text](url)", description: "Hyperlink" },
    SlashCommand { label: "Image", icon: "🖼", snippet: "![alt](url)", description: "Image embed" },
];

#[component]
pub fn Editor(state: Signal<AppState>, item: Item) -> Element {
    let mut content = use_signal(|| item.content.clone().unwrap_or_default());
    let mut title = use_signal(|| item.title.clone());
    let mut mode = use_signal(|| Mode::Split);
    let mut item_id = use_signal(|| item.id.clone());
    let mut show_slash = use_signal(|| false);
    let mut slash_filter = use_signal(|| String::new());
    let mut slash_selected = use_signal(|| 0usize);

    // Detect item switch — just load new content, NO save here.
    // Saving is handled by the sidebar BEFORE it switches items.
    let current_id = item.id.clone();
    if *item_id.read() != current_id {
        content.set(item.content.clone().unwrap_or_default());
        title.set(item.title.clone());
        item_id.set(current_id.clone());
        show_slash.set(false);
        // Register the new item's content as the baseline (not dirty)
        clear_editor_pending();
    }

    let current_mode = mode.read().clone();
    let slash_visible = *show_slash.read();
    let filter_text = slash_filter.read().clone().to_lowercase();

    let filtered_commands: Vec<&SlashCommand> = if filter_text.is_empty() {
        SLASH_COMMANDS.iter().collect()
    } else {
        SLASH_COMMANDS.iter()
            .filter(|c| c.label.to_lowercase().contains(&filter_text) || c.description.to_lowercase().contains(&filter_text))
            .collect()
    };

    let preview_html = if current_mode != Mode::Edit {
        render_markdown(&content.read())
    } else {
        String::new()
    };

    let word_count = content.read().split_whitespace().count();
    let line_count = content.read().lines().count();

    rsx! {
        div { class: "editor-container",
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
                div { class: "mode-toggle",
                    button {
                        class: if current_mode == Mode::Split { "active" } else { "" },
                        onclick: move |_| mode.set(Mode::Split),
                        "Split"
                    }
                    button {
                        class: if current_mode == Mode::Edit { "active" } else { "" },
                        onclick: move |_| mode.set(Mode::Edit),
                        "Edit"
                    }
                    button {
                        class: if current_mode == Mode::Preview { "active" } else { "" },
                        onclick: move |_| mode.set(Mode::Preview),
                        "Read"
                    }
                }
            }

            // Toolbar
            if current_mode != Mode::Preview {
                div { class: "editor-toolbar",
                    for cmd in SLASH_COMMANDS.iter().take(6) {
                        button {
                            class: "toolbar-btn",
                            title: "{cmd.description}",
                            onclick: {
                                let snippet = cmd.snippet.to_string();
                                move |_| {
                                    insert_snippet(&mut content, &snippet);
                                    let id = item_id.read().clone();
                                    set_editor_pending(&id, &content.read());
                                }
                            },
                            "{cmd.icon}"
                        }
                    }
                    span { class: "toolbar-divider" }
                    button { class: "toolbar-btn", title: "Code block", onclick: move |_| { insert_snippet(&mut content, "\n```\n\n```\n"); set_editor_pending(&item_id.read(), &content.read()); }, "<>" }
                    button { class: "toolbar-btn", title: "Table", onclick: move |_| { insert_snippet(&mut content, "\n| Col 1 | Col 2 |\n|--------|--------|\n| | |\n"); set_editor_pending(&item_id.read(), &content.read()); }, "▦" }
                    button { class: "toolbar-btn", title: "Divider", onclick: move |_| { insert_snippet(&mut content, "\n---\n"); set_editor_pending(&item_id.read(), &content.read()); }, "—" }
                    span { class: "toolbar-divider" }
                    button {
                        class: "toolbar-btn slash-trigger",
                        title: "All blocks (/)",
                        onclick: move |_| {
                            show_slash.set(!slash_visible);
                            slash_filter.set(String::new());
                            slash_selected.set(0);
                        },
                        "/"
                    }
                    span { class: "editor-stats", "{word_count}w · {line_count}L" }
                }
            }

            // Slash menu
            if slash_visible && current_mode != Mode::Preview {
                div { class: "slash-menu",
                    div { class: "slash-header",
                        input {
                            class: "slash-search",
                            placeholder: "Type to filter...",
                            value: "{slash_filter}",
                            oninput: move |e| { slash_filter.set(e.value()); slash_selected.set(0); },
                            onkeydown: move |e| {
                                let ft = slash_filter.read().to_lowercase();
                                let cmds: Vec<&SlashCommand> = SLASH_COMMANDS.iter()
                                    .filter(|c| ft.is_empty() || c.label.to_lowercase().contains(&ft) || c.description.to_lowercase().contains(&ft))
                                    .collect();
                                match e.key() {
                                    Key::ArrowDown => { let s = *slash_selected.read(); if s + 1 < cmds.len() { slash_selected.set(s + 1); } }
                                    Key::ArrowUp => { let s = *slash_selected.read(); if s > 0 { slash_selected.set(s - 1); } }
                                    Key::Enter => {
                                        if let Some(cmd) = cmds.get(*slash_selected.read()) {
                                            insert_snippet(&mut content, cmd.snippet);
                                            set_editor_pending(&item_id.read(), &content.read());
                                            show_slash.set(false);
                                            slash_filter.set(String::new());
                                        }
                                    }
                                    Key::Escape => { show_slash.set(false); slash_filter.set(String::new()); }
                                    _ => {}
                                }
                            },
                        }
                    }
                    div { class: "slash-list",
                        for (idx, cmd) in filtered_commands.iter().enumerate() {
                            div {
                                class: if idx == *slash_selected.read() { "slash-item selected" } else { "slash-item" },
                                onclick: {
                                    let snippet = cmd.snippet.to_string();
                                    move |_| {
                                        insert_snippet(&mut content, &snippet);
                                        set_editor_pending(&item_id.read(), &content.read());
                                        show_slash.set(false);
                                        slash_filter.set(String::new());
                                    }
                                },
                                span { class: "slash-icon", "{cmd.icon}" }
                                div { class: "slash-text",
                                    span { class: "slash-label", "{cmd.label}" }
                                    span { class: "slash-desc", "{cmd.description}" }
                                }
                            }
                        }
                        if filtered_commands.is_empty() {
                            p { class: "slash-empty", "No matching commands" }
                        }
                    }
                }
            }

            // Editor body
            div { class: "editor-body",
                if current_mode == Mode::Split || current_mode == Mode::Edit {
                    div { class: "editor-pane",
                        textarea {
                            class: "editor-textarea",
                            value: "{content}",
                            placeholder: "Start writing, or click / for blocks...",
                            oninput: move |e| {
                                let val = e.value();
                                if val.ends_with('/') {
                                    let before = &val[..val.len()-1];
                                    if before.is_empty() || before.ends_with('\n') || before.ends_with(' ') {
                                        show_slash.set(true);
                                        slash_filter.set(String::new());
                                        slash_selected.set(0);
                                    }
                                }
                                content.set(val);
                                // Register pending content for save-on-navigation
                                set_editor_pending(&item_id.read(), &content.read());
                            },
                            onblur: move |_| {
                                // Save on blur (explicit focus loss)
                                let text = content.read().clone();
                                let id = item_id.read().clone();
                                update_item_field(state, &id, None, Some(text), None, None);
                                clear_editor_pending();
                            },
                        }
                    }
                }
                if current_mode == Mode::Split || current_mode == Mode::Preview {
                    div {
                        class: "editor-preview",
                        dangerous_inner_html: "{preview_html}",
                    }
                }
            }
        }
    }
}

fn insert_snippet(content: &mut Signal<String>, snippet: &str) {
    let mut c = content.read().clone();
    if !c.is_empty() && !c.ends_with('\n') && (snippet.starts_with('\n') || snippet.starts_with('#') || snippet.starts_with('-') || snippet.starts_with('>') || snippet.starts_with('|')) {
        c.push('\n');
    }
    c.push_str(snippet);
    content.set(c);
}
