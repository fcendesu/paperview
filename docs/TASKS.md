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
| **Hybrid Theme (Dark/Cream)** | ✅ | ✅ | ✅ |
| **Live Reload (Watcher)** | ✅ | ✅ | ✅ |
| **Tabbed Interface** | ✅ | ✅ | ✅ |
| **Split View (Side-by-Side)** | ✅ | ✅ | ✅ |
| **History Sidebar** | ✅ | ✅ | ✅ |
| **Table of Contents (TOC)** | ✅ | ✅ | ✅ |
| **Scroll Synchronization** | ✅ | ✅ | ✅ |
| **In-Document Search** | ✅ | ✅ | ✅ |
| **LaTeX Support** | ✅ | ✅ | ✅ |
| **Mermaid Support** | ✅ | ✅ | ✅ |
| **Zen Mode** | ✅ | ✅ | ✅ |
| **Drag & Drop** | ✅ | ✅ | ✅ |

---

## MVP Notes

- History Sidebar includes shared persistence, stale-entry pruning, GUI click-to-open behavior, and the TUI recent-files dashboard.
- Hybrid Theme covers a shared config theme preference, GUI Iced styles, and TUI Ratatui style tokens.
- Live Reload covers active documents and visible split-pane secondary documents in both GUI and TUI; debouncing, background tab watching, and exact scroll restoration are deferred.
- Drag & Drop currently covers native GUI single-file and multi-file drops into tabs, plus a terminal-friendly TUI open-path prompt (`o`) for pasted or typed file paths.
- Zen Mode currently covers shared core state, the GUI focused reader layout, and the TUI full-width active reader layout; GUI and TUI preferences are persisted in config.
- Tabbed Interface currently covers shared open-document state, GUI tab activation and close controls, and TUI multi-file launch with `[` / `]` tab switching plus `x` tab close; reorder is deferred.
- Split View currently covers shared core split state, GUI comparison of the active tab with one other open tab, including a visible toggle, secondary tab selection, keyboard resizing, divider drag resizing, persisted split width, secondary pane scroll syncing, and secondary live reload, plus TUI side-by-side comparison toggled with `\`, secondary pane cycling with `{` / `}`, keyboard resizing with `<` / `>`, persisted split width, side-pane scroll syncing, and secondary live reload.
- Scroll Synchronization currently covers GUI active-reader TOC highlighting and click-to-scroll navigation based on estimated reader heading anchors, split-pane scroll syncing, plus TUI active-TOC highlighting, keyboard TOC jumps from rendered line anchors, and side-pane scroll syncing; exact GUI layout rectangles and mouse-based TUI TOC navigation are deferred.
- In-Document Search currently covers shared case-insensitive line search, GUI header query with previous/next controls, rendered-text highlighting with stronger selected-match emphasis, and TUI `/`, `n`, and `N` navigation with highlighted match lines; exact rendered-line geometry is deferred.
- Bookmarks currently cover shared core persistence plus TUI headless list/add/remove/prune commands and an interactive TUI picker that opens selected bookmarks near stored source lines; GUI sidebar integration and in-reader bookmark creation shortcuts are deferred.
- LaTeX Support covers the v0.1 foundation scope: structured inline math spans, a dedicated display math block, GUI, TUI, HTML export, and text-first PDF export readable previews for common display math tokens, Greek letters, arrows, set/logic operators, sums/integrals, compact scripts, source-preserving display math, and inline math export metadata; full formula typesetting is deferred.
- Mermaid Support covers the v0.1 foundation scope: parser recognition for `mermaid` fences, source-preserving diagram panels in GUI/TUI/export, and simple GUI/TUI/HTML-export flowchart previews with common labeled edge forms, comments, class suffixes, and common node shapes; full Mermaid layout, validation, and rich rendered assets are deferred.
- Table Rendering currently covers structured table parsing, inline cell formatting, GUI table panels with shared responsive column proportions, aligned TUI table output, and TUI long-cell wrapping; explicit wide-table scrolling is deferred.
- Task List Rendering currently covers checked and unchecked task markers in the shared parser model, GUI checkbox toggles with file writeback, and TUI file-backed task toggles from the current reader line; nested list structure is deferred.
- Image Rendering currently covers standalone image metadata blocks, inline image text preservation, GUI local and remote bitmap previews with metadata fallback and decoded PNG/JPEG/GIF/WebP dimensions, relative-path resolution, TUI Markdown image output with local/remote/missing metadata and local dimensions, and PDF image placeholders with local/remote/missing status plus local dimensions; dimension-driven layout and click-to-zoom are deferred.
- Inline Span Rendering currently covers heading, paragraph, list, blockquote, and table-cell bold, italic, inline code, and link metadata. GUI renders rich text with clickable links, including in-document heading anchors, and TUI renders Markdown-shaped inline text.

---

## CLI Toolkit (Headless)

| Command | Status | Notes |
| :--- | :---: | :--- |
| **paperview search** | ✅ | Prints ripgrep-backed path, line, column, and text results; `--interactive` opens a TUI result picker |
| **paperview export** | ✅ | v0.1 export covers standalone styled HTML with static Mermaid and readable math previews plus text-first PDF with basic layout, padded/wrapped tables, readable math preview text, heading outlines, and image metadata placeholders; rich PDF assets/renderers are deferred |
| **paperview tex compile/clean/doctor** | 🏗️ | Invokes the core Tectonic CLI adapter for a single `.tex` entry file, honors optional `tex_compiler_path`, writes the generated PDF under `.paperview/tex/`, reports diagnostics, supports `--open`, GUI external-open/reopen/clean, can clean file or directory artifacts, and can doctor compiler availability/version/smoke fixture status; bundling and embedded preview UI remain deferred |
| **paperview stats** | ✅ | Prints words, lines, characters, reading time, heading structure, and optional JSON |
| **paperview perf** | ✅ | Prints shape, deterministic memory estimate, target status, and baseline timings |
| **paperview config** | ✅ | Supports config path/edit plus theme, GUI/TUI Zen Mode, and Split View width preferences |

---

## Quality & Performance

| Task | Status | Target |
| :--- | :---: | :--- |
| **Cold Startup Time** | 🏗️ | 2026-05-26 local baselines remain under 10ms for headless document pipeline and GUI/TUI app-state startup; platform event-loop/window timing still needed |
| **Scrolling 60 FPS** | 🏗️ | `paperview-tui perf <file>` records deterministic rendered-line scroll workload; real frame timing still needed |
| **Memory Footprint** | 🏗️ | `paperview-tui perf docs/PRD.md` estimated 17.0KiB against the < 100MB MVP target on 2026-05-26 |
| **Zero-Dependency Build**| 🏗️ | Native Rust binaries with refreshed direct dependency and macOS arm64 release artifact baseline in `docs/quality/DEPENDENCIES.md`; release checklist added in `docs/quality/RELEASE_CHECKLIST.md`; Linux/Windows packaging checks still needed |

## Phase 2: Enhanced Functionality

| Feature | Core Logic | GUI (Iced) | TUI (Ratatui) |
| :--- | :---: | :---: | :---: |
| **Editing Mode** | ✅ | ✅ | ✅ |
| **Global Search** | ✅ | ✅ | ✅ |
| **Export HTML/PDF** | ✅ | N/A | ✅ |
| **Bookmarks** | 🏗️ | ⬜ | 🏗️ |

## Phase 3: Technical Advanced & Presentation

| Feature | Core Logic | GUI (Iced) | TUI (Ratatui) |
| :--- | :---: | :---: | :---: |
| **LaTeX/Math Foundation** | ✅ | ✅ | ✅ |
| **Tectonic `.tex` Support** | 🏗️ | 🏗️ | 🏗️ |
| **Presentation Mode** | ✅ | ✅ | ✅ |

## Deferred Out Of Current Roadmap Scope

- Full formula rendering/typesetting for Markdown math.
- Knowledge Graph visual connections between linked Markdown files.
