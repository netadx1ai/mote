// Mote Editor — Notion-like contenteditable with floating toolbar & slash commands
(function() {
    // --- Slash command definitions (B2B text icons, no emoji) ---
    const COMMANDS = [
        { label: 'Heading 1', icon: 'H1', tag: 'h1' },
        { label: 'Heading 2', icon: 'H2', tag: 'h2' },
        { label: 'Heading 3', icon: 'H3', tag: 'h3' },
        { label: 'Bullet List', icon: 'UL', tag: 'ul' },
        { label: 'Numbered List', icon: 'OL', tag: 'ol' },
        { label: 'Task List', icon: 'TL', html: '<ul><li><input type="checkbox"> </li></ul>' },
        { label: 'Quote', icon: 'Q', tag: 'blockquote' },
        { label: 'Code Block', icon: '{;}', tag: 'pre' },
        { label: 'Divider', icon: 'HR', tag: 'hr' },
    ];

    // --- Ensure editor elements exist (idempotent) ---
    function ensureEditor() {
        let wrap = document.getElementById('mote-editor-wrap');
        if (!wrap) {
            console.warn('[MoteEditor] mote-editor-wrap not found, retrying...');
            // Retry after a short delay
            setTimeout(function() { ensureEditor(); }, 100);
            return null;
        }
        // Make sure it's visible
        wrap.style.display = '';

        let editor = wrap.querySelector('.mote-ce-editor');
        if (editor) return editor;

        // Create contenteditable
        editor = document.createElement('div');
        editor.className = 'mote-ce-editor';
        editor.contentEditable = 'true';
        editor.setAttribute('data-placeholder', 'Start writing, or type / for commands...');
        wrap.appendChild(editor);

        // Attach event listeners
        editor.addEventListener('keydown', onKeyDown);
        editor.addEventListener('input', onInput);
        editor.addEventListener('blur', onBlur);

        return editor;
    }

    // --- Floating toolbar (created once) ---
    if (!document.getElementById('mote-float-tb')) {
        const toolbar = document.createElement('div');
        toolbar.id = 'mote-float-tb';
        toolbar.className = 'mote-float-toolbar';
        toolbar.innerHTML = `
            <button data-cmd="bold" title="Bold"><b>B</b></button>
            <button data-cmd="italic" title="Italic"><i>I</i></button>
            <button data-cmd="strikeThrough" title="Strike"><s>S</s></button>
            <span class="mote-tb-sep"></span>
            <button data-cmd="formatBlock" data-val="h1" title="Heading 1">H1</button>
            <button data-cmd="formatBlock" data-val="h2" title="Heading 2">H2</button>
            <button data-cmd="formatBlock" data-val="h3" title="Heading 3">H3</button>
            <span class="mote-tb-sep"></span>
            <button data-cmd="insertUnorderedList" title="Bullet list">UL</button>
            <button data-cmd="insertOrderedList" title="Numbered list">OL</button>
            <button data-cmd="formatBlock" data-val="blockquote" title="Quote">Q</button>
            <button data-cmd="formatBlock" data-val="pre" title="Code block">{;}</button>
            <button data-cmd="createLink" title="Insert link">Lk</button>
        `;
        toolbar.style.display = 'none';
        document.body.appendChild(toolbar);

        toolbar.addEventListener('mousedown', function(e) {
            e.preventDefault();
            const btn = e.target.closest('button');
            if (!btn) return;
            const cmd = btn.dataset.cmd;
            const val = btn.dataset.val || null;
            if (cmd === 'createLink') {
                const url = prompt('Enter URL:');
                if (url) document.execCommand('createLink', false, url);
            } else if (cmd === 'formatBlock') {
                document.execCommand('formatBlock', false, '<' + val + '>');
            } else {
                document.execCommand(cmd, false, val);
            }
            syncContent();
        });
    }

    // --- Slash menu (created once) ---
    if (!document.getElementById('mote-slash-m')) {
        const sm = document.createElement('div');
        sm.id = 'mote-slash-m';
        sm.className = 'mote-slash-menu';
        sm.style.display = 'none';
        document.body.appendChild(sm);

        sm.addEventListener('mousedown', function(e) {
            e.preventDefault();
            const item = e.target.closest('.mote-slash-item');
            if (item) {
                const idx = parseInt(item.dataset.idx);
                if (slashFiltered[idx]) applySlashCommand(slashFiltered[idx]);
            }
        });
    }

    // --- Selection toolbar ---
    if (!window.__moteSelHandler) {
        window.__moteSelHandler = true;
        document.addEventListener('selectionchange', function() {
            const toolbar = document.getElementById('mote-float-tb');
            if (!toolbar) return;
            const sel = window.getSelection();
            const editor = document.querySelector('.mote-ce-editor');
            if (!sel || sel.isCollapsed || !editor || !editor.contains(sel.anchorNode)) {
                toolbar.style.display = 'none';
                return;
            }
            const range = sel.getRangeAt(0);
            const rect = range.getBoundingClientRect();
            toolbar.style.display = 'flex';
            toolbar.style.top = (rect.top - 42 + window.scrollY) + 'px';
            toolbar.style.left = Math.max(8, rect.left + rect.width / 2 - toolbar.offsetWidth / 2) + 'px';
        });
    }

    // --- Slash state ---
    let slashActive = false;
    let slashIdx = 0;
    let slashFiltered = COMMANDS;

    function renderSlash(filter) {
        slashFiltered = COMMANDS.filter(c =>
            !filter || c.label.toLowerCase().includes(filter.toLowerCase())
        );
        slashIdx = 0;
        const sm = document.getElementById('mote-slash-m');
        if (!sm) return;
        if (slashFiltered.length === 0) {
            sm.innerHTML = '<div class="mote-slash-empty">No results</div>';
        } else {
            sm.innerHTML = slashFiltered.map((c, i) =>
                `<div class="mote-slash-item${i === slashIdx ? ' selected' : ''}" data-idx="${i}">
                    <span class="mote-slash-icon">${c.icon}</span>
                    <span class="mote-slash-label">${c.label}</span>
                </div>`
            ).join('');
        }
    }

    function showSlash() {
        slashActive = true;
        renderSlash('');
        const sel = window.getSelection();
        const sm = document.getElementById('mote-slash-m');
        if (sel && sel.rangeCount && sm) {
            const rect = sel.getRangeAt(0).getBoundingClientRect();
            sm.style.top = (rect.bottom + 4 + window.scrollY) + 'px';
            sm.style.left = rect.left + 'px';
            sm.style.display = 'block';
        }
    }

    function hideSlash() {
        slashActive = false;
        const sm = document.getElementById('mote-slash-m');
        if (sm) sm.style.display = 'none';
    }

    function highlightSlash() {
        const sm = document.getElementById('mote-slash-m');
        if (!sm) return;
        sm.querySelectorAll('.mote-slash-item').forEach(function(el, i) {
            el.classList.toggle('selected', i === slashIdx);
        });
    }

    function applySlashCommand(cmd) {
        const sel = window.getSelection();
        if (sel && sel.rangeCount) {
            const range = sel.getRangeAt(0);
            if (range.startOffset > 0) {
                range.setStart(range.startContainer, range.startOffset - 1);
                range.deleteContents();
            }
        }
        if (cmd.html) {
            document.execCommand('insertHTML', false, cmd.html);
        } else if (cmd.tag === 'hr') {
            document.execCommand('insertHTML', false, '<hr>');
        } else if (cmd.tag === 'ul' || cmd.tag === 'ol') {
            document.execCommand(cmd.tag === 'ul' ? 'insertUnorderedList' : 'insertOrderedList');
        } else if (cmd.tag === 'pre') {
            document.execCommand('formatBlock', false, '<pre>');
        } else {
            document.execCommand('formatBlock', false, '<' + cmd.tag + '>');
        }
        hideSlash();
        syncContent();
        const editor = document.querySelector('.mote-ce-editor');
        if (editor) editor.focus();
    }

    // --- Event handlers ---
    function onKeyDown(e) {
        if (slashActive) {
            if (e.key === 'ArrowDown') { e.preventDefault(); slashIdx = Math.min(slashIdx + 1, slashFiltered.length - 1); highlightSlash(); }
            else if (e.key === 'ArrowUp') { e.preventDefault(); slashIdx = Math.max(slashIdx - 1, 0); highlightSlash(); }
            else if (e.key === 'Enter') { e.preventDefault(); if (slashFiltered[slashIdx]) applySlashCommand(slashFiltered[slashIdx]); }
            else if (e.key === 'Escape') { e.preventDefault(); hideSlash(); }
            return;
        }
        if ((e.metaKey || e.ctrlKey) && !e.shiftKey) {
            if (e.key === 'b') { e.preventDefault(); document.execCommand('bold'); syncContent(); }
            if (e.key === 'i') { e.preventDefault(); document.execCommand('italic'); syncContent(); }
            if (e.key === 'u') { e.preventDefault(); document.execCommand('underline'); syncContent(); }
        }
    }

    function onInput() {
        const sel = window.getSelection();
        if (sel && sel.rangeCount && sel.isCollapsed) {
            const node = sel.anchorNode;
            if (node && node.nodeType === 3) {
                const text = node.textContent;
                const offset = sel.anchorOffset;
                const beforeCursor = text.substring(0, offset);
                const slashPos = beforeCursor.lastIndexOf('/');
                if (slashPos >= 0 && (slashPos === 0 || text[slashPos - 1] === ' ' || text[slashPos - 1] === '\n')) {
                    const filter = beforeCursor.substring(slashPos + 1);
                    if (!slashActive) showSlash();
                    renderSlash(filter);
                    return;
                }
            }
        }
        if (slashActive) hideSlash();
        syncContent();
    }

    function onBlur() {
        syncContent();
        const toolbar = document.getElementById('mote-float-tb');
        setTimeout(function() { if (toolbar) toolbar.style.display = 'none'; }, 200);
    }

    // --- Sync content to Rust via hidden input ---
    function syncContent() {
        const editor = document.querySelector('.mote-ce-editor');
        const bridge = document.getElementById('mote-content-bridge');
        if (editor && bridge) {
            const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
            setter.call(bridge, editor.innerHTML);
            bridge.dispatchEvent(new Event('input', { bubbles: true }));
        }
    }

    // --- API for Rust ---
    window.__moteEditor = {
        setContent: function(html) {
            const editor = ensureEditor();
            if (editor) editor.innerHTML = html || '';
        },
        getContent: function() {
            const editor = document.querySelector('.mote-ce-editor');
            return editor ? editor.innerHTML : '';
        },
    };

    window.__moteInitEditor = function(initialHtml) {
        window.__motePendingHtml = initialHtml || '';
        function tryInit() {
            const wrap = document.getElementById('mote-editor-wrap');
            if (!wrap) {
                setTimeout(tryInit, 50);
                return;
            }
            wrap.style.display = '';
            const editor = ensureEditor();
            if (editor) {
                editor.innerHTML = window.__motePendingHtml;
                editor.focus();
            } else {
                setTimeout(tryInit, 50);
            }
        }
        tryInit();
    };
})();
