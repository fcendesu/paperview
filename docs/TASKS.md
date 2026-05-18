# PaperView — Implementation Tracker

This document tracks the progress of features across the three workspace crates.

## Status Legend
- ⬜ **To-do**
- 🏗️ **In Progress**
- ✅ **Done**
- ❌ **Blocked**

---

## Phase 1: MVP (v0.1)

| Feature | Core Logic | GUI (Iced) | TUI (Ratatui) |
| :--- | :---: | :---: | :---: |
| **Project Setup** | ✅ | ✅ | ✅ |
| **File Opening (CLI/Open)** | ✅ | ✅ | ✅ |
| **Basic Markdown Rendering** | ✅ | ✅ | ✅ |
| **Table Rendering** | ✅ | ✅ | ✅ |
| **Task List Rendering** | ✅ | ✅ | ✅ |
| **Image Rendering** | ✅ | ✅ | ✅ |
| **Inline Span Rendering** | ✅ | ✅ | ✅ |
| **Hybrid Theme (Dark/Cream)** | ⬜ | ✅ | ⬜ |
| **Live Reload (Watcher)** | ✅ | ✅ | ✅ |
| **Tabbed Interface** | ✅ | ✅ | ⬜ |
| **Split View (Side-by-Side)** | ⬜ | ✅ | ⬜ |
| **History Sidebar** | ✅ | ✅ | ✅ |
| **Table of Contents (TOC)** | ✅ | ✅ | ✅ |
| **Scroll Synchronization** | ✅ | ✅ | ✅ |
| **In-Document Search** | ✅ | ✅ | ✅ |
| **LaTeX Support** | ✅ | 🏗️ | 🏗️ |
| **Mermaid Support** | ✅ | 🏗️ | 🏗️ |
| **Zen Mode** | ⬜ | ✅ | ⬜ |
| **Drag & Drop** | ✅ | ✅ | ⬜ |

---

## MVP Notes

- History Sidebar includes shared persistence, GUI click-to-open behavior, and the TUI recent-files dashboard.
- Live Reload covers active documents in both GUI and TUI; debouncing and exact scroll restoration are deferred.
- Drag & Drop currently covers native GUI single-file and multi-file drops into tabs.
- Zen Mode currently covers the GUI focused reader layout; TUI and persisted preferences are deferred.
- Tabbed Interface currently covers shared open-document state plus GUI tab activation and close controls; reorder/TUI tabs are deferred.
- Split View currently covers a GUI foundation for comparing the active tab with one other open tab, including a visible toggle, secondary tab selection, and keyboard resizing; drag resize, scroll sync, and TUI split are deferred.
- Scroll Synchronization currently covers GUI active-reader TOC highlighting and click-to-scroll navigation based on estimated reader heading anchors, plus TUI active-TOC highlighting and keyboard TOC jumps from rendered line anchors; exact GUI layout rectangles, split-pane scroll sync, and mouse-based TUI TOC navigation are deferred.
- In-Document Search currently covers shared case-insensitive line search, GUI header query with previous/next controls, and TUI `/`, `n`, and `N` navigation with highlighted match lines; GUI match highlighting and exact rendered-line geometry are deferred.
- LaTeX Support currently covers parser preservation for inline math, a dedicated display math block, GUI readable previews for common display math tokens, and TUI source-preserving display math; full formula typesetting and structured inline spans are deferred.
- Mermaid Support currently covers parser recognition for `mermaid` fences, source-preserving diagram panels in GUI/TUI, and simple GUI flowchart previews; full Mermaid layout, validation, TUI previews, and export assets are deferred.
- Table Rendering currently covers structured table parsing, inline cell formatting, GUI table panels, and aligned TUI table output; wide-table scrolling and responsive column sizing are deferred.
- Task List Rendering currently covers read-only checked and unchecked task markers in the shared parser model plus GUI/TUI rendering; interactive toggles and writeback are deferred.
- Image Rendering currently covers standalone image metadata blocks, inline image text preservation, GUI local bitmap previews with metadata fallback, relative-path resolution, and TUI Markdown image output; remote fetching, decoded-dimension layout, and click-to-zoom are deferred.
- Inline Span Rendering currently covers heading, paragraph, list, blockquote, and table-cell bold, italic, inline code, and link metadata. GUI renders rich text with clickable links, including in-document heading anchors, and TUI renders Markdown-shaped inline text.

---

## CLI Toolkit (Headless)

| Command | Status | Notes |
| :--- | :---: | :--- |
| **paperview search** | ⬜ | Needs `ripgrep` integration |
| **paperview export** | ⬜ | Needs PDF/HTML backend |
| **paperview stats** | ⬜ | Needs AST analysis logic |
| **paperview config** | ⬜ | Needs TOML persistence |

---

## Quality & Performance

| Task | Status | Target |
| :--- | :---: | :--- |
| **Cold Startup Time** | ⬜ | < 500ms |
| **Scrolling 60 FPS** | ⬜ | Constant |
| **Memory Footprint** | ⬜ | < 100MB (Typical) |
| **Zero-Dependency Build**| ⬜ | Single Binary |
