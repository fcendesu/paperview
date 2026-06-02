# PaperView — Technical Architecture (ARCH.md)

This document outlines the internal structure of PaperView, focusing on a **Developer-First** and **Ergonomic** implementation in Rust.

## 1. Ergonomic Principles (Developer-First)
- **Instant Response:** Every keypress and scroll must feel local. No async lag in the UI thread.
- **Keyboard-Centric:** Full navigation (`j`/`k` scrolling, `ctrl+p` file search) is a first-class citizen.
- **Context Preservation:** When a file is updated via `notify`, the scroll position and UI state must be preserved exactly.

---

## 2. Component Architecture
PaperView follows the **Elm Architecture (TEA)** provided by Iced:

### 2.1 State (Model)
```rust
struct PaperView {
    layout: LayoutMode,
    history: Vec<FileEntry>,
    bookmarks: Vec<Bookmark>, // Planned; not implemented yet.
    ui_state: UIState,
}

enum LayoutMode {
    Single(Document),
    Split {
        left: Document,
        right: Document,
        active_side: Side,
    },
    Tabbed {
        docs: Vec<Document>,
        active_index: usize,
    },
    Editor {
        doc: Document,
        buffer: String, // Raw text for editing (Phase 2)
    }
}
```

enum Side { Left, Right }
```

### 2.2 Messages & Commands
PaperView distinguishes between UI events and headless toolkit commands.

**UI Events:**
- `LayoutToggled(LayoutMode)`: Switches between Single, Split, or Tabbed views.
- `TabSelected(usize)`: Switches focus to a different open tab.
- `TabClosed(usize)`: Closes an open document.
- `FileOpened(PathBuf)`: Opens a new tab or switches to an existing one.
- `FileDropped(PathBuf)`: Triggered by system Drag & Drop event.
- `FileChanged(PathBuf)`: Triggered by the `notify` watcher.
- `ScrollOffsetChanged(f32)`: Syncs the Reader with the TOC.
- `OpenLink(String)`: Routes clicked GUI inline links. `#slug` targets scroll to
  matching active-document headings; other targets open through the platform
  default opener after resolving relative paths from the active document.
- **`MouseEvent(MouseAction)`**: Handles clicks and scroll-wheel input in both GUI and TUI.

**Headless Toolkit Commands:**
- `Search(String)`: Triggers `ripgrep` workspace search.
- `Export(Format)`: Headless PDF/HTML generation.
- `GetStats(PathBuf)`: Word count and document metadata analysis.
- `ManageConfig(Action)`: Reads or modifies the `config.toml`.
- `CompileTex(PathBuf)`: Invokes the configured Tectonic compiler and writes a
  PDF artifact for a `.tex` entry file.


---

## 3. The Rendering Pipeline
To achieve "Beautiful Typography" with `pulldown-cmark`:

1.  **Markdown Ingestion:** Read `.md` file into a string.
2.  **Event Parsing:** `pulldown-cmark` generates an event stream (Headings, Lists, etc.).
3.  **Widget Mapping:** A custom "Renderer" maps these events to Iced `Widget`s.
    - *Optimization:* Cache rendered widgets for large documents to maintain 60 FPS.
4.  **Styling:** Apply the "Hybrid" theme (Cream background, Serif fonts) via Iced `StyleSheet`.

---

## 4. Background Services (Subscriptions)
PaperView uses Iced `Subscription`s to handle non-blocking tasks:

- **File Watcher (`notify`):** Runs in a separate thread. Sends a `FileChanged` message when the current file is saved in another editor (like NeoVim or VSCode).
- **Keyboard Listener:** Captures global shortcuts (e.g., `Cmd+O`) even when the reader is focused.

The first live-reload implementation keeps watcher ownership in `paperview-core` and exposes frontend-neutral `WatchEvent` values. The GUI adapts those events into Iced subscriptions for the active document path.

The first tabs implementation keeps open-document ownership in
`paperview-core::OpenDocuments`. Frontends can use the shared model to add,
activate, and replace documents without duplicating path de-duplication rules.
The GUI uses this model for clickable tabs and close controls. The TUI uses the
same model for multi-file launch and keyboard tab switching with `[` / `]`;
`x` closes the active TUI tab through the shared close behavior. Its active tab
drives rendered lines, TOC, search results, and file watching.

The first LaTeX implementation keeps math semantics in `paperview-core` by
preserving inline math text and exposing display math as `Block::Math`.
Core also exposes a lightweight readable-preview transform for common display
math tokens. The GUI shows that preview above the preserved source when it can
improve readability; TUI output remains source-preserving.

The first Mermaid implementation keeps diagram semantics in `paperview-core` by
detecting `mermaid` fenced code blocks and exposing them as `Block::Diagram`.
Core also exposes a simple flowchart preview parser for common `graph` and
`flowchart` edge lists. The GUI renders those edges as native preview rows while
keeping source text visible; TUI output remains source-preserving.

The first table implementation keeps table structure in `paperview-core` by
consuming `pulldown-cmark` table events into `Block::Table`. Frontends render
from the shared alignments, header cells, and body rows instead of reparsing
Markdown text.

The first task-list implementation keeps checkbox state in `paperview-core` by
storing list items as `ListItem` values with optional checked state, source line
metadata, and inline content. Core owns source-line marker toggling through
`toggle_task_line_source`. The GUI uses that helper for file-backed checkbox
writeback and reloads the document after a successful write. The TUI continues
to render read-only checked and unchecked markers from the shared model.

The first image implementation keeps standalone image metadata in
`paperview-core` by promoting image-only paragraphs into `Block::Image`. Inline
images remain text until richer inline spans exist. The GUI resolves local
standalone image paths against the active document and renders bitmap previews
with Iced image widgets when the file exists.

The inline-span implementation stores `InlineSpan` values for heading,
paragraph, blockquote, list item, and table-cell content. Plain heading text is
derived for document titles, TOC labels, slugs, and scroll geometry. GUI rich
text attaches link metadata to inline link spans. Clicked `#slug` links reuse
the GUI TOC scroll path and update the active TOC item. Supported clicked local
document links resolve relative to the active document and open as PaperView
tabs; external and unsupported targets still delegate to the OS opener.

The first in-document search implementation keeps source search in
`paperview-core` through `Document::search` and line-based `SearchMatch` values.
GUI and TUI frontends own their local search state and use the shared matches to
jump reader scroll position.

The first workspace-search implementation keeps ripgrep invocation and result
parsing in `paperview-core::search_workspace`. The TUI binary exposes it as a
headless `search <query> [path]` command that prints path, line, column, and
matched text without initializing Ratatui.

The export implementation keeps format parsing and artifact creation in
`paperview-core`. `ExportFormat` recognizes HTML and PDF, `export_document`
returns completed artifacts, and `export_html` renders the shared parsed
document model while escaping user content before writing markup. HTML headings
use the same duplicate-safe slug sequence as the shared table of contents for
exported `id` anchors. `export_pdf` writes a dependency-light text-first PDF
from the same parsed document model. The TUI binary exposes this as a headless
`export <file> --to html|pdf` command that writes successful artifacts beside
the source document without initializing Ratatui.

The first document-stats implementation keeps metadata calculation in
`paperview-core` through `Document::stats` and `DocumentStats`. The TUI binary
exposes it as a headless `stats <file>` command that prints a report without
initializing Ratatui.

The first performance-baseline implementation lives in the TUI binary as a
headless `perf <file>` command. It measures source read time, shared document
parse/model construction, and TUI line rendering through
`render_document_with_anchors` without initializing Ratatui.

The first config implementation keeps TOML config path resolution and file
creation in `paperview-core::ConfigStore`. The shared config also owns the
optional Tectonic compiler path used by headless `.tex` compilation. The TUI
binary exposes headless `config path` and `config edit` commands without
initializing Ratatui.

The first `.tex` document path keeps LaTeX sources out of `Document::open`.
`paperview-core::compile_tex` owns Tectonic invocation and PDF artifact
validation. The TUI exposes this as `tex compile`; the GUI uses the same compile
path for `.tex` launch, drag-and-drop, and local links, then delegates the
generated PDF to the platform opener until embedded PDF preview exists.

The Editing Mode foundation keeps source-editing state in
`paperview-core::EditSession`. It owns the editable buffer, original source for
dirty-state checks, optional file path, preview document generation, and
file-backed save behavior. GUI and TUI frontends should render and mutate the
buffer while delegating save and preview semantics to core.

The Presentation Mode foundation keeps slide-boundary logic in
`paperview-core::presentation`. `PresentationDeck` turns Markdown source into
ordered `Slide` values, preferring explicit thematic-rule separators and
falling back to top-level headings. Slides preserve Markdown source so
frontends can reuse the existing document rendering pipeline.

Split View shared behavior lives in `paperview-core::SplitViewState`. It owns
the secondary-tab index, bounded primary-pane width, toggle/retarget rules, and
side-pane cycling. GUI and TUI frontends keep only presentation-specific state
such as cached rendered side-pane lines or mouse-drag cursor positions.

Zen Mode shared behavior lives in `paperview-core::ZenModeState`. It owns the
enabled/disabled state and toggle behavior while GUI and TUI frontends map that
state to their own layout chrome.

---

## 5. Performance Optimizations
- **Zero-Copy Parsing:** Use `pulldown-cmark`'s ability to reference the original string where possible.
- **Lazy Loading:** Only render the visible portion of the Markdown document for extremely long files.
- **Static Assets:** Embed fonts (JetBrains Mono, Inter) into the binary using `include_bytes!` for a zero-dependency "portable" feel.

---

## 6. Granular Directory Structure
To ensure "One Change = One File," we will use a highly modular structure.

### `paperview-core` (The Brain)
```text
src/
├── parser/
│   ├── mod.rs          # Parser orchestration
│   ├── engine.rs       # pulldown-cmark wrapper
│   └── elements/       # ONE FILE PER MARKDOWN ELEMENT
│       ├── mod.rs
│       ├── heading.rs
│       ├── table.rs
│       ├── math.rs     # LaTeX logic
│       └── diagram.rs  # Mermaid logic
├── watcher/            # notify-rs integration
├── document/           # Document & Tab models
└── state/              # History & Bookmark logic
```

### `paperview-gui` (The Iced Face)
```text
src/
├── ui/
│   ├── mod.rs
│   ├── theme/          # Hybrid, Dark, Light themes
│   ├── components/     # REUSABLE UI WIDGETS
│   │   ├── tab_bar.rs
│   │   ├── sidebar.rs
│   │   └── reader.rs
│   └── layout.rs       # Single, Split, Zen logic
└── main.rs             # Application Entry
```

### `paperview-tui` (The Ratatui Face)
```text
src/
├── term/
│   ├── mod.rs
│   ├── widgets/        # TUI-specific components
│   └── events.rs       # Key/Mouse event mapping
└── main.rs
```

## 7. Development Strategy: "Core-First, UI Slices"
To ensure feature parity and efficient development across both GUI and TUI frontends:

1.  **Core-First:** All business logic (parsing, file watching, history management, configuration) must be implemented in `paperview-core`. The frontends should be "thin" and only responsible for rendering and event capture.
2.  **Feature Slicing:** Development proceeds in horizontal slices. For each feature:
    - Implement logic in `paperview-core`.
    - Implement rendering in `paperview-gui` (Iced).
    - Implement rendering in `paperview-tui` (Ratatui).
3.  **State Synchronization:** Both frontends must share the same configuration and history files (managed by `core`) to ensure a seamless transition between the terminal and the desktop.
