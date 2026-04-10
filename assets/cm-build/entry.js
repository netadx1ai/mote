import {
  EditorView, keymap, drawSelection, placeholder,
  lineNumbers, highlightActiveLine, highlightActiveLineGutter,
} from "@codemirror/view";
import { EditorState, Compartment } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { languages } from "@codemirror/language-data";
import { syntaxHighlighting, HighlightStyle, bracketMatching } from "@codemirror/language";
import { autocompletion, closeBrackets } from "@codemirror/autocomplete";
import { tags } from "@lezer/highlight";

const MONO = "'JetBrains Mono','Fira Code','Menlo','Consolas',monospace";

// VS Code Dark+ inspired colors
const mdHighlightDark = HighlightStyle.define([
  { tag: tags.heading1,   fontSize: "1.75em", fontWeight: "700", color: "#569cd6" },
  { tag: tags.heading2,   fontSize: "1.4em",  fontWeight: "700", color: "#569cd6" },
  { tag: tags.heading3,   fontSize: "1.15em", fontWeight: "600", color: "#569cd6" },
  { tag: tags.heading4,   fontSize: "1.05em", fontWeight: "600", color: "#569cd6" },
  { tag: tags.strong,     fontWeight: "700",  color: "#dcdcaa" },
  { tag: tags.emphasis,   fontStyle: "italic", color: "#ce9178" },
  { tag: tags.strikethrough, textDecoration: "line-through", opacity: "0.6" },
  { tag: tags.link,       color: "#4ec9b0", textDecoration: "underline" },
  { tag: tags.url,        color: "#4ec9b0" },
  { tag: tags.monospace,  fontFamily: MONO, fontSize: "0.875em", color: "#ce9178" },
  { tag: tags.processingInstruction, color: "#6a9955" },
  { tag: tags.punctuation, color: "#888" },
  { tag: tags.quote,      color: "#6a9955", fontStyle: "italic" },
  { tag: tags.list,       color: "#c586c0" },
]);

const mdHighlightLight = HighlightStyle.define([
  { tag: tags.heading1,   fontSize: "1.75em", fontWeight: "700", color: "#0000ff" },
  { tag: tags.heading2,   fontSize: "1.4em",  fontWeight: "700", color: "#0070c1" },
  { tag: tags.heading3,   fontSize: "1.15em", fontWeight: "600", color: "#0070c1" },
  { tag: tags.heading4,   fontSize: "1.05em", fontWeight: "600", color: "#0070c1" },
  { tag: tags.strong,     fontWeight: "700",  color: "#795e26" },
  { tag: tags.emphasis,   fontStyle: "italic", color: "#a31515" },
  { tag: tags.strikethrough, textDecoration: "line-through", opacity: "0.6" },
  { tag: tags.link,       color: "#267f99", textDecoration: "underline" },
  { tag: tags.url,        color: "#267f99" },
  { tag: tags.monospace,  fontFamily: MONO, fontSize: "0.875em", color: "#a31515" },
  { tag: tags.processingInstruction, color: "#008000" },
  { tag: tags.punctuation, color: "#aaa" },
  { tag: tags.quote,      color: "#008000", fontStyle: "italic" },
  { tag: tags.list,       color: "#af00db" },
]);

const themeCompartment = new Compartment();
let view = null;
let _programmatic = false;

// ── Floating selection toolbar ──────────────────────────────────────────────
let toolbar = null;

function ensureToolbar() {
  if (toolbar) return toolbar;
  toolbar = document.createElement("div");
  toolbar.id = "cm-float-toolbar";
  toolbar.style.cssText = [
    "position:fixed", "z-index:9999", "display:none",
    "background:#1e1e1e", "border:1px solid #3c3c3c", "border-radius:6px",
    "padding:4px 6px", "gap:2px", "align-items:center",
    "box-shadow:0 4px 16px rgba(0,0,0,0.4)",
    "font-family:" + MONO, "font-size:12px",
  ].join(";");

  const btns = [
    { label: "B",  title: "Bold (⌘B)",   pre: "**", post: "**", css: "font-weight:700" },
    { label: "I",  title: "Italic (⌘I)", pre: "_",  post: "_",  css: "font-style:italic" },
    { label: "`",  title: "Code (⌘`)",   pre: "`",  post: "`",  css: "font-family:monospace" },
    { label: "~~", title: "Strikethrough", pre: "~~", post: "~~", css: "text-decoration:line-through" },
    { label: "🔗", title: "Link",         pre: "[",  post: "](url)", css: "" },
  ];

  for (const btn of btns) {
    const el = document.createElement("button");
    el.textContent = btn.label;
    el.title = btn.title;
    el.style.cssText = [
      "background:transparent", "border:none", "color:#ccc",
      "padding:3px 8px", "border-radius:4px", "cursor:pointer", btn.css,
    ].join(";");
    el.addEventListener("mouseenter", () => el.style.background = "#3c3c3c");
    el.addEventListener("mouseleave", () => el.style.background = "transparent");
    el.addEventListener("mousedown", (e) => {
      e.preventDefault();
      if (!view) return;
      wrapSelection(view, btn.pre, btn.post);
      view.focus();
      hideToolbar();
    });
    toolbar.appendChild(el);
  }

  document.body.appendChild(toolbar);
  return toolbar;
}

function showToolbar(x, y) {
  const tb = ensureToolbar();
  tb.style.display = "flex";
  tb.style.left = Math.max(8, x - 20) + "px";
  tb.style.top  = (y - 46) + "px";
}
function hideToolbar() {
  if (toolbar) toolbar.style.display = "none";
}
function updateToolbar() {
  if (!view) return hideToolbar();
  const sel = view.state.selection.main;
  if (sel.empty) return hideToolbar();
  const coords = view.coordsAtPos(sel.head);
  if (!coords) return hideToolbar();
  showToolbar(coords.left, coords.top);
}

// ── Theme packs ──────────────────────────────────────────────────────────────
function lightExts() {
  return [
    EditorView.theme({
      "&":        { height: "100%", fontSize: "14px", fontFamily: MONO, background: "#ffffff", color: "#1f2937" },
      ".cm-scroller": { overflow: "auto", height: "100%", lineHeight: "1.65" },
      ".cm-content":  { padding: "16px 20px 80px", minHeight: "100%", caretColor: "#1f2937" },
      ".cm-focused":  { outline: "none" },
      ".cm-cursor":   { borderLeftColor: "#374151", borderLeftWidth: "2px" },
      ".cm-selectionBackground": { background: "#add6ff !important" },
      ".cm-activeLine":      { background: "rgba(0,0,0,0.04)" },
      ".cm-activeLineGutter": { background: "rgba(0,0,0,0.04)" },
      ".cm-gutters": {
        background: "#f9fafb", borderRight: "1px solid #e5e7eb",
        color: "#9ca3af", minWidth: "44px",
      },
      ".cm-lineNumbers .cm-gutterElement": { padding: "0 12px 0 8px" },
      ".cm-placeholder": { color: "#9ca3af" },
    }, { dark: false }),
    syntaxHighlighting(mdHighlightLight),
  ];
}

function darkExts() {
  return [
    EditorView.theme({
      "&":        { height: "100%", fontSize: "14px", fontFamily: MONO, background: "#1e1e1e", color: "#d4d4d4" },
      ".cm-scroller": { overflow: "auto", height: "100%", lineHeight: "1.65" },
      ".cm-content":  { padding: "16px 20px 80px", minHeight: "100%", caretColor: "#d4d4d4" },
      ".cm-focused":  { outline: "none" },
      ".cm-cursor":   { borderLeftColor: "#aeafad", borderLeftWidth: "2px" },
      ".cm-selectionBackground": { background: "#264f78 !important" },
      ".cm-activeLine":      { background: "rgba(255,255,255,0.04)" },
      ".cm-activeLineGutter": { background: "rgba(255,255,255,0.04)" },
      ".cm-gutters": {
        background: "#252526", borderRight: "1px solid #3c3c3c",
        color: "#858585", minWidth: "44px",
      },
      ".cm-lineNumbers .cm-gutterElement": { padding: "0 12px 0 8px" },
      ".cm-placeholder": { color: "#6b7280" },
    }, { dark: true }),
    syntaxHighlighting(mdHighlightDark),
  ];
}

// ── Helpers ───────────────────────────────────────────────────────────────────
function wrapSelection(v, pre, post) {
  const sel = v.state.selection.main;
  if (sel.empty) {
    v.dispatch({
      changes: { from: sel.from, insert: pre + post },
      selection: { anchor: sel.from + pre.length },
    });
  } else {
    const text = v.state.sliceDoc(sel.from, sel.to);
    v.dispatch({
      changes: { from: sel.from, to: sel.to, insert: pre + text + post },
      selection: { anchor: sel.from + pre.length, head: sel.from + pre.length + text.length },
    });
  }
}

function triggerBridge(id, value) {
  const el = document.getElementById(id);
  if (!el) return;
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
  setter.call(el, value);
  el.dispatchEvent(new Event("input", { bubbles: true }));
}

// ── Public API ────────────────────────────────────────────────────────────────
window.moteEditor = {
  init(container, initialContent, isDark) {
    if (view) { view.destroy(); view = null; }
    if (!container) return;

    const state = EditorState.create({
      doc: initialContent || "",
      extensions: [
        history(),
        lineNumbers(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        drawSelection(),
        bracketMatching(),
        closeBrackets(),
        autocompletion({ activateOnTyping: false }),
        EditorView.lineWrapping,
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          indentWithTab,
          {
            key: "Mod-s",
            run() { triggerBridge("cm-save-bridge", Date.now().toString()); return true; },
          },
          {
            key: "Mod-b",
            run(v) { wrapSelection(v, "**", "**"); return true; },
          },
          {
            key: "Mod-i",
            run(v) { wrapSelection(v, "_", "_"); return true; },
          },
        ]),
        markdown({ base: markdownLanguage, codeLanguages: languages }),
        placeholder("Start writing…"),
        EditorView.updateListener.of((update) => {
          if (update.docChanged && !_programmatic) {
            triggerBridge("cm-content-bridge", update.state.doc.toString());
          }
          if (update.selectionSet) updateToolbar();
        }),
        EditorView.domEventHandlers({
          blur() { triggerBridge("cm-save-bridge", Date.now().toString()); },
        }),
        themeCompartment.of(isDark ? darkExts() : lightExts()),
      ],
    });

    view = new EditorView({ state, parent: container });
    view.dom.addEventListener("click", updateToolbar);

    document.addEventListener("mousedown", (e) => {
      if (toolbar && !toolbar.contains(e.target)) hideToolbar();
    }, { capture: true });

    view.focus();
  },

  setContent(text) {
    if (!view) return;
    if (view.state.doc.toString() === text) return;
    _programmatic = true;
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text ?? "" } });
    requestAnimationFrame(() => { _programmatic = false; });
  },

  setTheme(isDark) {
    if (!view) return;
    view.dispatch({ effects: themeCompartment.reconfigure(isDark ? darkExts() : lightExts()) });
  },

  // Used by toolbar buttons in editor.rs
  insertText(prefix, suffix) {
    if (!view) return;
    wrapSelection(view, prefix, suffix || "");
    view.focus();
  },

  insertBlock(prefix) {
    if (!view) return;
    const pos = view.state.selection.main.from;
    const line = view.state.doc.lineAt(pos);
    const stripped = line.text.replace(/^#{1,6}\s*/, "");
    view.dispatch({
      changes: { from: line.from, to: line.to, insert: prefix + stripped },
      selection: { anchor: line.from + prefix.length + stripped.length },
    });
    view.focus();
  },

  focus() { view?.focus(); },
  destroy() { view?.destroy(); view = null; hideToolbar(); },
};
