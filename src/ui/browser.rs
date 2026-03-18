use dioxus::prelude::*;
use dioxus::desktop::{window, Config, tao::window::WindowBuilder};

#[derive(Clone, PartialEq, Props)]
pub struct BrowserWindowProps {
    url: String,
}

/// Browser window component — full-page iframe that fills the entire native window.
/// This is a dedicated window, so the iframe IS the browser experience.
fn BrowserPage(props: BrowserWindowProps) -> Element {
    let url = props.url.clone();

    rsx! {
        style { "
            * {{ margin: 0; padding: 0; box-sizing: border-box; }}
            html, body {{ height: 100%; overflow: hidden; background: #191919; }}
            .browser-frame {{ width: 100%; height: 100%; border: none; }}
            .browser-bar {{
                display: flex; align-items: center; gap: 8px; padding: 6px 10px;
                background: #202020; font-family: -apple-system, sans-serif;
                border-bottom: 1px solid #333; height: 36px;
            }}
            .browser-bar .url {{
                flex: 1; font-size: 12px; color: #999; overflow: hidden;
                text-overflow: ellipsis; white-space: nowrap;
                font-family: 'SF Mono', monospace;
            }}
            .browser-bar button {{
                background: #333; color: #ccc; border: none; padding: 4px 10px;
                border-radius: 4px; font-size: 11px; cursor: pointer;
                font-family: inherit;
            }}
            .browser-bar button:hover {{ background: #444; color: #fff; }}
            .content {{ height: calc(100% - 36px); }}
        " }
        div { class: "browser-bar",
            span { class: "url", "{url}" }
            button {
                onclick: move |_| {
                    // Reload iframe
                    dioxus::document::eval("document.querySelector('.browser-frame').src = document.querySelector('.browser-frame').src;");
                },
                "Reload"
            }
            button {
                onclick: {
                    let u = url.clone();
                    move |_| {
                        // Copy URL to clipboard
                        let js = format!("navigator.clipboard.writeText('{}')", u.replace('\'', "\\'"));
                        dioxus::document::eval(&js);
                    }
                },
                "Copy URL"
            }
        }
        div { class: "content",
            iframe {
                class: "browser-frame",
                src: "{props.url}",
            }
        }
    }
}

/// Open a URL in a new native webview window.
pub fn open_in_browser_window(url: &str) {
    let url = normalize_url(url);
    let title = url_to_title(&url);

    let dom = VirtualDom::new_with_props(
        BrowserPage,
        BrowserWindowProps { url },
    );

    let cfg = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title(&title)
                .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1100.0, 750.0))
        );

    window().new_window(dom, cfg);
}

fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "https://duckduckgo.com".to_string();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    if trimmed.contains('.') && !trimmed.contains(' ') {
        return format!("https://{trimmed}");
    }
    format!("https://duckduckgo.com/?q={}", trimmed.replace(' ', "+"))
}

fn url_to_title(url: &str) -> String {
    url.split("//")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or("Browser")
        .to_string()
}

/// Browser panel in the main app — URL bar + quick links.
#[component]
pub fn BrowserView() -> Element {
    let mut url_input = use_signal(|| String::new());
    let mut history = use_signal(|| Vec::<String>::new());

    let mut do_navigate = move |_: ()| {
        let url = url_input.read().clone();
        if !url.trim().is_empty() {
            let normalized = normalize_url(&url);
            open_in_browser_window(&normalized);
            let mut h = history.write();
            h.retain(|u| u != &normalized);
            h.insert(0, normalized);
            if h.len() > 20 { h.truncate(20); }
        }
    };

    let hist = history.read().clone();

    rsx! {
        div { class: "browser-panel",
            h2 { "Web Browser" }
            p { class: "browser-hint", "Opens pages in native browser windows" }

            div { class: "browser-url-bar",
                input {
                    class: "browser-url",
                    placeholder: "Enter URL or search term...",
                    value: "{url_input}",
                    oninput: move |e| url_input.set(e.value()),
                    onkeypress: move |e| {
                        if e.key() == Key::Enter { do_navigate(()); }
                    },
                }
                button {
                    class: "btn-primary",
                    style: "padding: 8px 20px; flex-shrink: 0;",
                    onclick: move |_| do_navigate(()),
                    "Open"
                }
            }

            div { class: "browser-quick",
                h3 { "Quick Links" }
                div { class: "quick-links",
                    for (label, url) in [
                        ("DuckDuckGo", "https://duckduckgo.com"),
                        ("Wikipedia", "https://en.wikipedia.org"),
                        ("MDN Docs", "https://developer.mozilla.org"),
                        ("Rust Docs", "https://doc.rust-lang.org"),
                        ("GitHub", "https://github.com"),
                        ("Hacker News", "https://news.ycombinator.com"),
                    ] {
                        button {
                            class: "quick-link",
                            onclick: move |_| open_in_browser_window(url),
                            "{label}"
                        }
                    }
                }
            }

            if !hist.is_empty() {
                div { class: "browser-history",
                    h3 { "Recent" }
                    for url in hist.iter() {
                        div {
                            class: "history-item",
                            onclick: {
                                let u = url.clone();
                                move |_| open_in_browser_window(&u)
                            },
                            "{url}"
                        }
                    }
                }
            }
        }
    }
}
