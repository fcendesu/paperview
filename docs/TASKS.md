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
| **Hybrid Theme (Dark/Cream)** | ⬜ | ✅ | ✅ |
| **Live Reload (Watcher)** | ✅ | ✅ | ✅ |
| **Tabbed Interface** | ✅ | ✅ | ✅ |
| **Split View (Side-by-Side)** | ⬜ | ✅ | ✅ |
| **History Sidebar** | ✅ | ✅ | ✅ |
| **Table of Contents (TOC)** | ✅ | ✅ | ✅ |
| **Scroll Synchronization** | ✅ | ✅ | ✅ |
| **In-Document Search** | ✅ | ✅ | ✅ |
| **LaTeX Support** | ✅ | 🏗️ | 🏗️ |
| **Mermaid Support** | ✅ | 🏗️ | 🏗️ |
| **Zen Mode** | ⬜ | ✅ | ✅ |
| **Drag & Drop** | ✅ | ✅ | ⬜ |

---

## MVP Notes

- History Sidebar includes shared persistence, GUI click-to-open behavior, and the TUI recent-files dashboard.
- Hybrid Theme covers GUI Iced styles and TUI Ratatui style tokens; core has no theme ownership.
- Live Reload covers active documents in both GUI and TUI; debouncing and exact scroll restoration are deferred.
- Drag & Drop currently covers native GUI single-file and multi-file drops into tabs.
- Zen Mode currently covers the GUI focused reader layout and the TUI full-width active reader layout; persisted preferences are deferred.
- Tabbed Interface currently covers shared open-document state, GUI tab activation and close controls, and TUI multi-file launch with `[` / `]` tab switching plus `x` tab close; reorder is deferred.
- Split View currently covers GUI comparison of the active tab with one other open tab, including a visible toggle, secondary tab selection, keyboard resizing, and divider drag resizing, plus TUI side-by-side comparison toggled with `\`, secondary pane cycling with `{` / `}`, and keyboard resizing with `<` / `>`; scroll sync is deferred.
- Scroll Synchronization currently covers GUI active-reader TOC highlighting and click-to-scroll navigation based on estimated reader heading anchors, plus TUI active-TOC highlighting and keyboard TOC jumps from rendered line anchors; exact GUI layout rectangles, split-pane scroll sync, and mouse-based TUI TOC navigation are deferred.
- In-Document Search currently covers shared case-insensitive line search, GUI header query with previous/next controls, rendered-text highlighting with stronger selected-match emphasis, and TUI `/`, `n`, and `N` navigation with highlighted match lines; exact rendered-line geometry is deferred.
- LaTeX Support currently covers parser preservation for inline math, a dedicated display math block, GUI and TUI readable previews for common display math tokens, and source-preserving display math; full formula typesetting and structured inline spans are deferred.
- Mermaid Support currently covers parser recognition for `mermaid` fences, source-preserving diagram panels in GUI/TUI, and simple GUI/TUI flowchart previews; full Mermaid layout, validation, and export assets are deferred.
- Table Rendering currently covers structured table parsing, inline cell formatting, GUI table panels, and aligned TUI table output; wide-table scrolling and responsive column sizing are deferred.
- Task List Rendering currently covers checked and unchecked task markers in the shared parser model, GUI checkbox toggles with file writeback, and TUI file-backed task toggles from the current reader line; nested list structure is deferred.
- Image Rendering currently covers standalone image metadata blocks, inline image text preservation, GUI local and remote bitmap previews with metadata fallback, relative-path resolution, and TUI Markdown image output; decoded-dimension layout and click-to-zoom are deferred.
- Inline Span Rendering currently covers heading, paragraph, list, blockquote, and table-cell bold, italic, inline code, and link metadata. GUI renders rich text with clickable links, including in-document heading anchors, and TUI renders Markdown-shaped inline text.

---

## CLI Toolkit (Headless)

| Command | Status | Notes |
| :--- | :---: | :--- |
| **paperview search** | ✅ | Prints ripgrep-backed path, line, column, and text results |
| **paperview export** | 🏗️ | HTML export writes standalone styled `.html`; PDF export writes a basic text-first `.pdf` |
| **paperview stats** | ✅ | Prints words, lines, characters, reading time, and heading structure |
| **paperview perf** | ✅ | Prints shape, deterministic memory estimate, target status, and baseline timings |
| **paperview config** | ✅ | Supports config path and edit with TOML file creation |

---

## Quality & Performance

| Task | Status | Target |
| :--- | :---: | :--- |
| **Cold Startup Time** | 🏗️ | `paperview-tui perf <file>` records read/parse/render baseline and load target status; full interactive startup timing still needed |
| **Scrolling 60 FPS** | ⬜ | Constant |
| **Memory Footprint** | 🏗️ | `paperview-tui perf <file>` estimates source/model/rendered text payloads against < 100MB |
| **Zero-Dependency Build**| ⬜ | Single Binary |
