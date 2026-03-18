use dioxus::prelude::*;
use std::path::{Path, PathBuf};

const MONACO_JS: &str = include_str!("monaco_init.js");

/// A single open file tab
#[derive(Clone, PartialEq)]
pub struct FileTab {
    pub path: PathBuf,
    pub content: String,
    pub dirty: bool,
}

/// Directory entry for the tree (shallow — no recursive children)
#[derive(Clone, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// Scan a single directory level — no recursion
fn scan_dir_shallow(path: &Path) -> Vec<DirEntry> {
    let Ok(read) = std::fs::read_dir(path) else {
        return vec![];
    };
    let mut entries: Vec<DirEntry> = read
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e.path();
            let is_dir = path.is_dir();
            Some(DirEntry { name, path, is_dir })
        })
        .collect();
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

fn is_text_file(path: &Path) -> bool {
    if let Ok(meta) = path.metadata() {
        if meta.len() > 10 * 1024 * 1024 {
            return false;
        }
    }
    let text_exts = [
        "txt", "md", "rs", "toml", "json", "yaml", "yml", "xml", "html", "htm",
        "css", "js", "ts", "tsx", "jsx", "py", "rb", "go", "java", "c", "cpp",
        "h", "hpp", "sh", "bash", "zsh", "fish", "sql", "graphql", "proto",
        "env", "gitignore", "dockerignore", "dockerfile", "makefile", "cmake",
        "lock", "cfg", "ini", "conf", "log", "csv", "tsv", "svg", "ron",
        "editorconfig", "prettierrc", "eslintrc", "babelrc",
    ];
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        text_exts.contains(&ext_lower.as_str())
    } else {
        let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        matches!(name.as_str(),
            "makefile" | "dockerfile" | "readme" | "license" | "changelog" |
            "authors" | "todo" | ".gitignore" | ".gitattributes" | ".editorconfig" |
            ".env" | ".env.local" | ".env.example" | ".prettierrc" | ".eslintrc"
        )
    }
}

/// Map file extension to Monaco language ID
fn ext_to_language(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("toml") => "toml",
        Some("json") => "json",
        Some("yaml" | "yml") => "yaml",
        Some("xml") => "xml",
        Some("html" | "htm") => "html",
        Some("css") => "css",
        Some("js" | "jsx") => "javascript",
        Some("ts" | "tsx") => "typescript",
        Some("py") => "python",
        Some("rb") => "ruby",
        Some("go") => "go",
        Some("java") => "java",
        Some("c" | "h") => "c",
        Some("cpp" | "hpp") => "cpp",
        Some("sh" | "bash" | "zsh") => "shell",
        Some("sql") => "sql",
        Some("md") => "markdown",
        Some("graphql") => "graphql",
        Some("dockerfile") => "dockerfile",
        Some("svg") => "xml",
        Some("ini" | "cfg" | "conf") => "ini",
        Some("csv" | "tsv") => "plaintext",
        Some("log") => "plaintext",
        _ => "plaintext",
    }
}

/// Returns (icon_char, css_color) — kept minimal: folder vs file
fn file_icon(_path: &Path, is_dir: bool) -> (&'static str, &'static str) {
    if is_dir {
        ("\u{25B8}", "rgba(255,255,255,0.3)")
    } else {
        ("\u{2012}", "rgba(255,255,255,0.2)")  // figure dash = file
    }
}

// --- Monaco bridge ---

fn init_monaco() {
    let js = format!("setTimeout(function(){{{MONACO_JS}}},0);");
    dioxus::document::eval(&js);
}

fn mount_monaco(content: &str, language: &str, tab_id: &str) {
    let escaped_content = content.replace('\\', "\\\\").replace('`', "\\`").replace("${", "\\${");
    let escaped_id = tab_id.replace('\\', "\\\\").replace('`', "\\`");
    let js = format!(
        "setTimeout(function(){{window.__moteMonacoMount(`{escaped_content}`,'{language}',`{escaped_id}`);}},100);"
    );
    dioxus::document::eval(&js);
}

fn switch_monaco_tab(content: &str, language: &str, tab_id: &str) {
    let escaped_content = content.replace('\\', "\\\\").replace('`', "\\`").replace("${", "\\${");
    let escaped_id = tab_id.replace('\\', "\\\\").replace('`', "\\`");
    let js = format!(
        "setTimeout(function(){{window.__moteMonacoSetContent(`{escaped_content}`,'{language}',`{escaped_id}`);}},0);"
    );
    dioxus::document::eval(&js);
}

// --- Sidebar component ---

#[component]
pub fn FileExplorerSidebar(
    workspace_root: PathBuf,
    tabs: Signal<Vec<FileTab>>,
    active_tab: Signal<Option<usize>>,
    browse_root: Signal<Option<PathBuf>>,
) -> Element {
    // Derive effective root reactively from the signal
    let root = browse_root.read().clone().unwrap_or_else(|| workspace_root.clone());
    let root_for_new = root.clone();
    let root_display = root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string());
    // Scan only top level — children loaded lazily on expand
    let entries = scan_dir_shallow(&root);

    rsx! {
        div { class: "section-header",
            span { "{root_display}" }
            div { style: "display: flex; gap: 2px;",
                button {
                    class: "add-btn",
                    title: "Open folder",
                    onclick: move |_| {
                        pick_folder(browse_root);
                    },
                    "\u{2026}"
                }
                button {
                    class: "add-btn",
                    title: "New file",
                    onclick: move |_| {
                        create_new_file(root_for_new.clone(), tabs, active_tab);
                    },
                    "+"
                }
            }
        }
        div { class: "fe-tree",
            for entry in entries.iter() {
                FileTreeNode { entry: entry.clone(), depth: 0, tabs, active_tab }
            }
            if entries.is_empty() {
                p { class: "empty", "No files in this directory" }
            }
        }
    }
}

fn pick_folder(mut browse_root: Signal<Option<PathBuf>>) {
    let picked = rfd::FileDialog::new()
        .set_title("Open folder")
        .pick_folder();
    if let Some(path) = picked {
        browse_root.set(Some(path));
    }
}

fn create_new_file(root: PathBuf, mut tabs: Signal<Vec<FileTab>>, mut active_tab: Signal<Option<usize>>) {
    let mut name = "untitled.md".to_string();
    let mut path = root.join(&name);
    let mut i = 1;
    while path.exists() {
        name = format!("untitled-{i}.md");
        path = root.join(&name);
        i += 1;
    }
    if std::fs::write(&path, "").is_ok() {
        let tab = FileTab {
            path,
            content: String::new(),
            dirty: false,
        };
        let mut t = tabs.write();
        t.push(tab);
        let idx = t.len() - 1;
        drop(t);
        active_tab.set(Some(idx));
    }
}

// --- Tree node ---

#[component]
fn FileTreeNode(
    entry: DirEntry,
    depth: i32,
    tabs: Signal<Vec<FileTab>>,
    mut active_tab: Signal<Option<usize>>,
) -> Element {
    let mut expanded = use_signal(|| false);
    let mut children = use_signal(|| Vec::<DirEntry>::new());
    let padding = format!("padding-left: {}px;", 8 + depth * 16);
    let entry_path = entry.path.clone();
    let is_dir = entry.is_dir;
    let is_expanded = *expanded.read();

    let is_active = tabs.read().iter().enumerate().any(|(i, t)| {
        t.path == entry_path && active_tab.read().as_ref() == Some(&i)
    });

    let node_class = if is_active {
        "tree-node active"
    } else if is_dir {
        "tree-node tree-node-dir"
    } else {
        "tree-node tree-node-file"
    };
    let child_count = children.read().len();

    rsx! {
        div { class: "tree-node-wrap",
            div {
                class: "{node_class}",
                style: "{padding}",
                onclick: {
                    let path = entry.path.clone();
                    move |_| {
                        if is_dir {
                            let cur = *expanded.read();
                            if !cur {
                                // Lazy-load children on first expand
                                children.set(scan_dir_shallow(&path));
                            }
                            expanded.set(!cur);
                        } else {
                            open_file(path.clone(), tabs, active_tab);
                        }
                    }
                },
                if is_dir {
                    span { class: "fe-chevron",
                        style: if is_expanded { "transform: rotate(90deg);" } else { "" },
                        "\u{25B8}"
                    }
                }
                span { class: "node-title", "{entry.name}" }
                if is_dir && is_expanded && child_count > 0 {
                    span { class: "node-child-count", "{child_count}" }
                }
            }
        }
        if is_dir && is_expanded {
            for child in children.read().iter() {
                FileTreeNode { entry: child.clone(), depth: depth + 1, tabs, active_tab }
            }
        }
    }
}

fn open_file(path: PathBuf, mut tabs: Signal<Vec<FileTab>>, mut active_tab: Signal<Option<usize>>) {
    let existing = tabs.read().iter().position(|t| t.path == path);
    if let Some(idx) = existing {
        active_tab.set(Some(idx));
        return;
    }

    if !is_text_file(&path) {
        return;
    }

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let tab = FileTab {
        path,
        content,
        dirty: false,
    };
    let mut t = tabs.write();
    t.push(tab);
    let idx = t.len() - 1;
    drop(t);
    active_tab.set(Some(idx));
}

// --- Main view ---

#[component]
pub fn FileExplorerView(
    tabs: Signal<Vec<FileTab>>,
    active_tab: Signal<Option<usize>>,
    browse_root: PathBuf,
) -> Element {
    // Track previous tab to detect switches
    let mut prev_tab_id = use_signal(|| String::new());
    let mut monaco_inited = use_signal(|| false);

    let active_idx = active_tab.read().clone();
    let tab_list: Vec<(usize, String, bool)> = tabs.read().iter().enumerate().map(|(i, t)| {
        let name = t.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "untitled".into());
        (i, name, t.dirty)
    }).collect();
    let root_display = browse_root.to_string_lossy().to_string();

    // Init Monaco once — use effect so it runs after DOM commit
    use_effect(move || {
        if !*monaco_inited.read() {
            monaco_inited.set(true);
            init_monaco();
        }
    });

    // Mount/switch Monaco when active tab changes — must run after DOM commit
    use_effect(move || {
        let active_idx = active_tab.read().clone();
        if let Some(idx) = active_idx {
            let t = tabs.read();
            if let Some(tab) = t.get(idx) {
                let tab_id = tab.path.to_string_lossy().to_string();
                let prev = prev_tab_id.read().clone();
                if prev != tab_id {
                    let language = ext_to_language(&tab.path).to_string();
                    let content = tab.content.clone();
                    drop(t);
                    if prev.is_empty() {
                        mount_monaco(&content, &language, &tab_id);
                    } else {
                        switch_monaco_tab(&content, &language, &tab_id);
                    }
                    prev_tab_id.set(tab_id);
                }
            }
        }
    });

    rsx! {
        div { class: "fe-container",
            // Breadcrumb
            div { class: "fe-path-bar",
                span { class: "fe-path-icon", "\u{25B7}" }
                span { class: "fe-path-text", "{root_display}" }
            }

            // Tab bar
            if !tab_list.is_empty() {
                div { class: "fe-tab-bar",
                    for (idx, name, dirty) in tab_list.iter() {
                        {
                            let idx = *idx;
                            let is_active = active_idx == Some(idx);
                            let tab_class = if is_active { "fe-tab active" } else { "fe-tab" };
                            let display_name = if *dirty { format!("{name} \u{2022}") } else { name.clone() };
                            rsx! {
                                div {
                                    class: "{tab_class}",
                                    onclick: move |_| active_tab.set(Some(idx)),
                                    span { class: "fe-tab-name", "{display_name}" }
                                    button {
                                        class: "fe-tab-close",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            close_tab(tabs, active_tab, idx);
                                        },
                                        "\u{00D7}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Editor area
            if let Some(idx) = active_idx {
                {
                    let tab_exists = idx < tabs.read().len();
                    if tab_exists {
                        let tab = tabs.read()[idx].clone();
                        let file_path_display = tab.path.to_string_lossy().to_string();
                        let ext = tab.path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
                        rsx! {
                            div { class: "fe-editor-header",
                                span { class: "fe-file-path", "{file_path_display}" }
                                div { class: "fe-editor-actions",
                                    if !ext.is_empty() {
                                        span { class: "fe-lang-badge", "{ext}" }
                                    }
                                    button {
                                        id: "monaco-save-btn",
                                        class: "btn-secondary",
                                        onclick: move |_| {
                                            // Read content from Monaco before saving
                                            sync_monaco_to_tab(tabs, idx);
                                            save_tab(tabs, idx);
                                        },
                                        "Save"
                                    }
                                    button {
                                        class: "btn-danger",
                                        title: "Delete file",
                                        onclick: move |_| {
                                            delete_file(tabs, active_tab, idx);
                                        },
                                        "Del"
                                    }
                                }
                            }
                            // Hidden bridge input for Monaco -> Dioxus sync
                            input {
                                id: "monaco-bridge",
                                r#type: "hidden",
                                oninput: move |e| {
                                    let val = e.value();
                                    let mut t = tabs.write();
                                    if let Some(tab) = t.get_mut(idx) {
                                        tab.content = val;
                                        tab.dirty = true;
                                    }
                                },
                            }
                            // Monaco container
                            div {
                                id: "monaco-container",
                                class: "fe-monaco-container",
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "empty-state",
                                p { "Tab not found" }
                            }
                        }
                    }
                }
            } else {
                div { class: "empty-state",
                    h2 { "File Explorer" }
                    p { "Open a file from the sidebar to start editing." }
                    p { class: "hint", "Click [\u{2026}] in the sidebar to browse a different folder." }
                }
            }
        }
    }
}

fn sync_monaco_to_tab(_tabs: Signal<Vec<FileTab>>, _idx: usize) {
    // Trigger Monaco to push content to bridge
    dioxus::document::eval(
        "if(window.__moteMonaco&&window.__moteMonaco.editor){var b=document.getElementById('monaco-bridge');if(b){var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(b,window.__moteMonaco.editor.getValue());b.dispatchEvent(new Event('input',{bubbles:true}));}}"
    );
}

fn save_tab(mut tabs: Signal<Vec<FileTab>>, idx: usize) {
    let t = tabs.read();
    if let Some(tab) = t.get(idx) {
        let path = tab.path.clone();
        let content = tab.content.clone();
        drop(t);
        if std::fs::write(&path, &content).is_ok() {
            if let Some(tab) = tabs.write().get_mut(idx) {
                tab.dirty = false;
            }
        }
    }
}

fn close_tab(mut tabs: Signal<Vec<FileTab>>, mut active_tab: Signal<Option<usize>>, idx: usize) {
    let mut t = tabs.write();
    if idx < t.len() {
        t.remove(idx);
        let len = t.len();
        drop(t);
        if len == 0 {
            active_tab.set(None);
        } else {
            let current = active_tab.read().unwrap_or(0);
            if current >= len {
                active_tab.set(Some(len - 1));
            } else if current > idx {
                active_tab.set(Some(current - 1));
            }
            let cur_active = *active_tab.read();
            if cur_active.map(|i| i >= len).unwrap_or(false) {
                active_tab.set(Some(len.saturating_sub(1)));
            }
        }
    }
}

fn delete_file(tabs: Signal<Vec<FileTab>>, active_tab: Signal<Option<usize>>, idx: usize) {
    let t = tabs.read();
    if let Some(tab) = t.get(idx) {
        let path = tab.path.clone();
        drop(t);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        close_tab(tabs, active_tab, idx);
    }
}
