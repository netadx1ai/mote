// Monaco Editor initializer for Mote file explorer
// Loaded via include_str! and eval'd in Dioxus webview

(function() {
    // Prevent double-init
    if (window.__moteMonaco && window.__moteMonaco.editor) {
        return;
    }

    window.__moteMonaco = { editor: null, ready: false };

    // Load Monaco from CDN if not already loaded
    if (!window.require) {
        var loaderScript = document.createElement('script');
        loaderScript.src = 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs/loader.min.js';
        loaderScript.onload = function() {
            initMonaco();
        };
        document.head.appendChild(loaderScript);
    } else {
        initMonaco();
    }

    function initMonaco() {
        require.config({
            paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs' }
        });

        require(['vs/editor/editor.main'], function() {
            // Define Mote dark theme
            monaco.editor.defineTheme('mote-dark', {
                base: 'vs-dark',
                inherit: true,
                rules: [
                    { token: 'comment', foreground: '5c6370', fontStyle: 'italic' },
                    { token: 'keyword', foreground: 'c678dd' },
                    { token: 'string', foreground: '98c379' },
                    { token: 'number', foreground: 'd19a66' },
                    { token: 'type', foreground: 'e5c07b' },
                    { token: 'function', foreground: '61afef' },
                    { token: 'variable', foreground: 'e06c75' },
                ],
                colors: {
                    'editor.background': '#191919',
                    'editor.foreground': '#d4d4d4',
                    'editor.lineHighlightBackground': '#1e1e1e',
                    'editor.selectionBackground': '#264f78',
                    'editorCursor.foreground': '#528bff',
                    'editorLineNumber.foreground': '#3a3a3a',
                    'editorLineNumber.activeForeground': '#606060',
                    'editor.inactiveSelectionBackground': '#1e3a5f',
                    'editorIndentGuide.background1': '#2a2a2a',
                    'editorWidget.background': '#252525',
                    'editorWidget.border': '#333333',
                    'editorSuggestWidget.background': '#252525',
                    'editorSuggestWidget.border': '#333333',
                    'editorSuggestWidget.selectedBackground': '#333333',
                    'scrollbar.shadow': '#00000000',
                    'scrollbarSlider.background': 'rgba(255,255,255,0.08)',
                    'scrollbarSlider.hoverBackground': 'rgba(255,255,255,0.15)',
                    'scrollbarSlider.activeBackground': 'rgba(255,255,255,0.2)',
                }
            });

            window.__moteMonaco.ready = true;
            // If there's a pending mount, do it now
            if (window.__moteMonaco.pendingMount) {
                var p = window.__moteMonaco.pendingMount;
                window.__moteMonaco.pendingMount = null;
                window.__moteMonacoMount(p.content, p.language, p.tabId);
            }
        });
    }
})();

// Mount editor into #monaco-container with given content & language
window.__moteMonacoMount = function(content, language, tabId) {
    if (!window.__moteMonaco.ready) {
        window.__moteMonaco.pendingMount = { content: content, language: language, tabId: tabId };
        return;
    }

    var container = document.getElementById('monaco-container');
    if (!container) return;

    // Dispose previous editor
    if (window.__moteMonaco.editor) {
        window.__moteMonaco.editor.dispose();
        window.__moteMonaco.editor = null;
    }

    var editor = monaco.editor.create(container, {
        value: content,
        language: language,
        theme: 'mote-dark',
        automaticLayout: true,
        minimap: { enabled: false },
        fontSize: 13,
        lineHeight: 21,
        fontFamily: "'SF Mono', 'JetBrains Mono', 'Fira Code', monospace",
        fontLigatures: true,
        renderLineHighlight: 'line',
        scrollBeyondLastLine: false,
        smoothScrolling: true,
        cursorBlinking: 'smooth',
        cursorSmoothCaretAnimation: 'on',
        padding: { top: 12 },
        wordWrap: 'off',
        tabSize: 4,
        insertSpaces: true,
        bracketPairColorization: { enabled: true },
        guides: { indentation: true, bracketPairs: true },
        scrollbar: {
            verticalScrollbarSize: 8,
            horizontalScrollbarSize: 8,
        },
    });

    window.__moteMonaco.editor = editor;
    window.__moteMonaco.currentTabId = tabId;

    // Sync changes back to Dioxus via bridge input
    editor.onDidChangeModelContent(function() {
        var bridge = document.getElementById('monaco-bridge');
        if (bridge) {
            var setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
            setter.call(bridge, editor.getValue());
            bridge.dispatchEvent(new Event('input', { bubbles: true }));
        }
    });

    // Cmd+S / Ctrl+S save
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, function() {
        var saveBtn = document.getElementById('monaco-save-btn');
        if (saveBtn) saveBtn.click();
    });
};

// Update content without recreating editor (for tab switches)
window.__moteMonacoSetContent = function(content, language, tabId) {
    if (!window.__moteMonaco || !window.__moteMonaco.editor) {
        window.__moteMonacoMount(content, language, tabId);
        return;
    }
    if (window.__moteMonaco.currentTabId === tabId) {
        return; // Same tab, no update needed
    }
    var editor = window.__moteMonaco.editor;
    var model = editor.getModel();
    if (model) {
        monaco.editor.setModelLanguage(model, language);
        model.setValue(content);
    }
    window.__moteMonaco.currentTabId = tabId;
};

// Get current editor content
window.__moteMonacoGetContent = function() {
    if (window.__moteMonaco && window.__moteMonaco.editor) {
        return window.__moteMonaco.editor.getValue();
    }
    return '';
};
