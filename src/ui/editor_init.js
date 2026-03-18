// Mote Editor — Notion-like contenteditable with floating toolbar & slash commands
(function() {
    // --- Slash command definitions (B2B text icons, no emoji) ---
    const COMMANDS = [
        { label: 'Heading 1', icon: 'H\u2081', tag: 'h1' },
        { label: 'Heading 2', icon: 'H\u2082', tag: 'h2' },
        { label: 'Heading 3', icon: 'H\u2083', tag: 'h3' },
        { label: 'Bullet List', icon: '\u2022', tag: 'ul' },
        { label: 'Numbered List', icon: '\u2630', tag: 'ol' },
        { label: 'Task List', icon: '\u2610', html: '<ul><li><input type="checkbox"> </li></ul>' },
        { label: 'Quote', icon: '\u275D', tag: 'blockquote' },
        { label: 'Code Block', icon: '\u2774\u2775', tag: 'pre' },
        { label: 'Divider', icon: '\u2500', tag: 'hr' },
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
            <button data-cmd="formatBlock" data-val="p" title="Normal text">P</button>
            <button data-cmd="formatBlock" data-val="h1" title="Heading 1">H&#8321;</button>
            <button data-cmd="formatBlock" data-val="h2" title="Heading 2">H&#8322;</button>
            <button data-cmd="formatBlock" data-val="h3" title="Heading 3">H&#8323;</button>
            <span class="mote-tb-sep"></span>
            <button data-cmd="insertUnorderedList" title="Bullet list">&#8226;</button>
            <button data-cmd="insertOrderedList" title="Numbered list">&#9776;</button>
            <button data-cmd="formatBlock" data-val="blockquote" title="Quote">&#10077;</button>
            <button data-cmd="formatBlock" data-val="pre" title="Code block">&#10100;&#10101;</button>
            <button data-cmd="createLink" title="Insert link">&#128279;</button>
            <span class="mote-tb-sep"></span>
            <button class="mote-color-trigger" data-type="fg" title="Text color"><span style="color:#d4d4d4">A</span><span class="mote-color-bar" style="background:#d4d4d4"></span></button>
            <button class="mote-color-trigger" data-type="bg" title="Background color"><span class="mote-bg-icon">A</span><span class="mote-color-bar" style="background:#fef08a"></span></button>
            <span class="mote-tb-sep"></span>
            <button data-cmd="removeFormat" title="Clear formatting">&#8709;</button>
        `;
        toolbar.style.display = 'none';
        document.body.appendChild(toolbar);

        // --- Color picker panel ---
        const colorPanel = document.createElement('div');
        colorPanel.id = 'mote-color-panel';
        colorPanel.className = 'mote-color-panel';
        colorPanel.style.display = 'none';
        document.body.appendChild(colorPanel);

        const COLORS = [
            { label: 'White', val: '#ffffff' },
            { label: 'Light gray', val: '#a0a0a0' },
            { label: 'Red', val: '#ef4444' },
            { label: 'Orange', val: '#f59e0b' },
            { label: 'Yellow', val: '#eab308' },
            { label: 'Green', val: '#22c55e' },
            { label: 'Blue', val: '#3b82f6' },
            { label: 'Purple', val: '#a855f7' },
            { label: 'Pink', val: '#ec4899' },
        ];
        const BG_COLORS = [
            { label: 'None', val: 'transparent' },
            { label: 'Yellow', val: '#fef08a' },
            { label: 'Green', val: '#bbf7d0' },
            { label: 'Blue', val: '#bfdbfe' },
            { label: 'Purple', val: '#e9d5ff' },
            { label: 'Pink', val: '#fce7f3' },
            { label: 'Red', val: '#fecaca' },
            { label: 'Orange', val: '#fed7aa' },
            { label: 'Gray', val: '#404040' },
        ];

        let colorPanelType = 'fg'; // 'fg' or 'bg'
        let savedSelection = null;

        function saveSelection() {
            const sel = window.getSelection();
            if (sel && sel.rangeCount) savedSelection = sel.getRangeAt(0).cloneRange();
        }
        function restoreSelection() {
            if (savedSelection) {
                const sel = window.getSelection();
                sel.removeAllRanges();
                sel.addRange(savedSelection);
            }
        }

        function showColorPanel(type, anchor) {
            colorPanelType = type;
            const colors = type === 'fg' ? COLORS : BG_COLORS;
            colorPanel.innerHTML = colors.map(function(c) {
                var swatch = c.val === 'transparent'
                    ? '<span class="mote-swatch mote-swatch-none" title="' + c.label + '"></span>'
                    : '<span class="mote-swatch" style="background:' + c.val + '" title="' + c.label + '"></span>';
                return '<button data-color="' + c.val + '">' + swatch + '</button>';
            }).join('');
            var rect = anchor.getBoundingClientRect();
            colorPanel.style.display = 'flex';
            colorPanel.style.top = (rect.bottom + 4 + window.scrollY) + 'px';
            colorPanel.style.left = rect.left + 'px';
        }

        function hideColorPanel() {
            colorPanel.style.display = 'none';
        }

        colorPanel.addEventListener('mousedown', function(e) {
            e.preventDefault();
            e.stopPropagation();
            var btn = e.target.closest('button');
            if (!btn) return;
            var color = btn.dataset.color;
            restoreSelection();
            if (colorPanelType === 'fg') {
                document.execCommand('foreColor', false, color);
                // Update the bar color on the trigger button
                var fgTrigger = toolbar.querySelector('.mote-color-trigger[data-type="fg"] .mote-color-bar');
                if (fgTrigger) fgTrigger.style.background = color;
            } else {
                if (color === 'transparent') {
                    document.execCommand('hiliteColor', false, 'transparent');
                } else {
                    document.execCommand('hiliteColor', false, color);
                }
                var bgTrigger = toolbar.querySelector('.mote-color-trigger[data-type="bg"] .mote-color-bar');
                if (bgTrigger) bgTrigger.style.background = color === 'transparent' ? '#555' : color;
            }
            hideColorPanel();
            syncContent();
        });

        // Hide color panel on outside click
        document.addEventListener('mousedown', function(e) {
            if (!colorPanel.contains(e.target) && !e.target.closest('.mote-color-trigger')) {
                hideColorPanel();
            }
        });

        toolbar.addEventListener('mousedown', function(e) {
            e.preventDefault();
            // Handle color trigger buttons
            var colorTrigger = e.target.closest('.mote-color-trigger');
            if (colorTrigger) {
                saveSelection();
                var type = colorTrigger.dataset.type;
                if (colorPanel.style.display !== 'none' && colorPanelType === type) {
                    hideColorPanel();
                } else {
                    showColorPanel(type, colorTrigger);
                }
                return;
            }
            const btn = e.target.closest('button');
            if (!btn) return;
            const cmd = btn.dataset.cmd;
            const val = btn.dataset.val || null;
            if (cmd === 'createLink') {
                const url = prompt('Enter URL:');
                if (url) document.execCommand('createLink', false, url);
            } else if (cmd === 'removeFormat') {
                // Clear inline formatting (bold, italic, underline, strike, etc.)
                document.execCommand('removeFormat', false, null);
                // Also unlink
                document.execCommand('unlink', false, null);
                // Reset block to plain paragraph
                document.execCommand('formatBlock', false, '<p>');
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

    // --- Block handle (hover grip, drag to reorder, click arrows) ---
    if (!document.getElementById('mote-block-handle')) {
        const handle = document.createElement('div');
        handle.id = 'mote-block-handle';
        handle.className = 'mote-block-handle';
        handle.innerHTML =
            '<button data-dir="up" title="Move up">&#9652;</button>' +
            '<div class="mote-bh-grip" title="Drag to reorder">&#8942;&#8942;</div>' +
            '<button data-dir="down" title="Move down">&#9662;</button>';
        handle.style.display = 'none';
        document.body.appendChild(handle);

        // Drop indicator line between blocks
        const dropLine = document.createElement('div');
        dropLine.id = 'mote-block-dropline';
        dropLine.className = 'mote-block-dropline';
        dropLine.style.display = 'none';
        document.body.appendChild(dropLine);

        let hoveredBlock = null;
        let dragBlock = null;

        // --- Click up/down ---
        handle.addEventListener('mousedown', function(e) {
            const btn = e.target.closest('button');
            if (!btn || !hoveredBlock) return;
            e.preventDefault();
            moveBlock(hoveredBlock, btn.dataset.dir);
            setTimeout(function() { positionHandle(hoveredBlock); }, 10);
        });

        // --- Mouse-based drag from grip ---
        const grip = handle.querySelector('.mote-bh-grip');
        let dropTarget = null;
        let dropAbove = true;

        grip.addEventListener('mousedown', function(e) {
            if (!hoveredBlock) return;
            e.preventDefault();
            e.stopPropagation();
            dragBlock = hoveredBlock;
            dragBlock.classList.add('mote-dragging-block');
            handle.style.display = 'none';

            function onMouseMove(ev) {
                const editor = document.querySelector('.mote-ce-editor');
                if (!editor || !dragBlock) return;
                const target = document.elementFromPoint(ev.clientX, ev.clientY);
                const block = target ? getTopBlock(target) : null;
                if (block && block !== dragBlock && editor.contains(block)) {
                    const rect = block.getBoundingClientRect();
                    const midY = rect.top + rect.height / 2;
                    dropAbove = ev.clientY < midY;
                    const lineY = dropAbove ? rect.top : rect.bottom;
                    const editorRect = editor.getBoundingClientRect();
                    dropLine.style.display = 'block';
                    dropLine.style.top = (lineY + window.scrollY) + 'px';
                    dropLine.style.left = editorRect.left + 'px';
                    dropLine.style.width = editorRect.width + 'px';
                    dropTarget = block;
                } else {
                    dropLine.style.display = 'none';
                    dropTarget = null;
                }
            }

            function onMouseUp() {
                document.removeEventListener('mousemove', onMouseMove);
                document.removeEventListener('mouseup', onMouseUp);
                const editor = document.querySelector('.mote-ce-editor');
                if (dragBlock && dropTarget && editor) {
                    if (dropAbove) {
                        editor.insertBefore(dragBlock, dropTarget);
                    } else {
                        if (dropTarget.nextSibling) {
                            editor.insertBefore(dragBlock, dropTarget.nextSibling);
                        } else {
                            editor.appendChild(dragBlock);
                        }
                    }
                    syncContent();
                }
                if (dragBlock) dragBlock.classList.remove('mote-dragging-block');
                dragBlock = null;
                dropTarget = null;
                dropLine.style.display = 'none';
            }

            document.addEventListener('mousemove', onMouseMove);
            document.addEventListener('mouseup', onMouseUp);
        });

        function getTopBlock(el) {
            const editor = document.querySelector('.mote-ce-editor');
            if (!editor || !el) return null;
            let node = el;
            while (node && node.parentElement !== editor) {
                node = node.parentElement;
                if (!node) return null;
            }
            return node;
        }

        function positionHandle(block) {
            if (!block || !block.parentElement) { handle.style.display = 'none'; return; }
            const rect = block.getBoundingClientRect();
            handle.style.display = 'flex';
            handle.style.top = (rect.top + window.scrollY) + 'px';
            handle.style.left = Math.max(0, rect.left - 34) + 'px';
            handle.style.height = Math.min(rect.height, 48) + 'px';
        }

        let hideTimer = null;

        function showHandle(block) {
            if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
            hoveredBlock = block;
            positionHandle(block);
        }

        function hideHandle() {
            if (hideTimer) return;
            hideTimer = setTimeout(function() {
                handle.style.display = 'none';
                hoveredBlock = null;
                hideTimer = null;
            }, 200);
        }

        // Keep handle visible when hovering the handle itself
        handle.addEventListener('mouseenter', function() {
            if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
        });
        handle.addEventListener('mouseleave', function() {
            hideHandle();
        });

        document.addEventListener('mousemove', function(e) {
            if (dragBlock) return;
            const editor = document.querySelector('.mote-ce-editor');
            if (!editor) return;
            const target = document.elementFromPoint(e.clientX, e.clientY);
            if (!target) return;
            if (handle.contains(target)) return;
            if (editor.contains(target)) {
                const block = getTopBlock(target);
                if (block) {
                    showHandle(block);
                    return;
                }
            }
            hideHandle();
        });
    }

    function moveBlock(block, dir) {
        const editor = document.querySelector('.mote-ce-editor');
        if (!editor || !block) return;
        if (dir === 'up' && block.previousElementSibling) {
            editor.insertBefore(block, block.previousElementSibling);
        } else if (dir === 'down' && block.nextElementSibling) {
            editor.insertBefore(block.nextElementSibling, block);
        }
        syncContent();
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
        // Escape from block elements with Enter:
        // - blockquote: Enter at end exits to new paragraph
        // - pre (code block): Enter at end when last line is empty exits (like Notion double-enter)
        if (e.key === 'Enter' && !e.shiftKey) {
            const sel = window.getSelection();
            if (sel && sel.isCollapsed && sel.rangeCount) {
                const node = sel.anchorNode;
                const block = node.nodeType === 1 ? node : node.parentElement;
                const container = block ? block.closest('pre, blockquote') : null;
                if (container) {
                    const range = sel.getRangeAt(0);
                    const testRange = document.createRange();
                    testRange.selectNodeContents(container);
                    testRange.setStart(range.endContainer, range.endOffset);
                    const textAfter = testRange.toString();
                    const isAtEnd = textAfter === '' || textAfter === '\n';

                    if (isAtEnd) {
                        const isPre = container.tagName === 'PRE';
                        if (isPre) {
                            // Code block: only escape if last line is already empty (double-enter)
                            const text = container.textContent || '';
                            const lastNewline = text.lastIndexOf('\n');
                            const lastLine = lastNewline >= 0 ? text.substring(lastNewline + 1) : text;
                            if (lastLine.trim() !== '') {
                                // Last line has content — just insert normal newline, don't escape
                                return;
                            }
                            // Last line is empty — escape out, remove the trailing empty line
                            container.textContent = text.substring(0, lastNewline >= 0 ? lastNewline : 0);
                        }

                        e.preventDefault();
                        const p = document.createElement('p');
                        p.innerHTML = '<br>';
                        container.after(p);
                        const newRange = document.createRange();
                        newRange.setStart(p, 0);
                        newRange.collapse(true);
                        sel.removeAllRanges();
                        sel.addRange(newRange);
                        syncContent();
                        return;
                    }
                }
            }
        }
        // Alt+Up/Down: move block
        if (e.altKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
            e.preventDefault();
            const sel = window.getSelection();
            if (sel && sel.rangeCount) {
                const node = sel.anchorNode.nodeType === 1 ? sel.anchorNode : sel.anchorNode.parentElement;
                const editor = document.querySelector('.mote-ce-editor');
                if (node && editor) {
                    let block = node;
                    while (block && block.parentElement !== editor) block = block.parentElement;
                    if (block) {
                        moveBlock(block, e.key === 'ArrowUp' ? 'up' : 'down');
                        // Restore cursor in the moved block
                        const range = document.createRange();
                        range.selectNodeContents(block);
                        range.collapse(false);
                        sel.removeAllRanges();
                        sel.addRange(range);
                    }
                }
            }
            return;
        }
        // Keyboard shortcuts
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
    // Ensure content ends with a paragraph so user can always type after blocks
    function ensureTrailingParagraph(editor) {
        if (!editor) return;
        const last = editor.lastElementChild;
        if (!last || last.tagName === 'PRE' || last.tagName === 'BLOCKQUOTE' || last.tagName === 'HR' || last.tagName === 'TABLE' || last.tagName === 'UL' || last.tagName === 'OL') {
            const p = document.createElement('p');
            p.innerHTML = '<br>';
            editor.appendChild(p);
        }
    }

    window.__moteEditor = {
        setContent: function(html) {
            const editor = ensureEditor();
            if (editor) { editor.innerHTML = html || ''; ensureTrailingParagraph(editor); }
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
                ensureTrailingParagraph(editor);
                editor.focus();
            } else {
                setTimeout(tryInit, 50);
            }
        }
        tryInit();
    };
})();
