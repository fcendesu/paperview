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
    bookmarks: Vec<Bookmark>,
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
- **`MouseEvent(MouseAction)`**: Handles clicks and scroll-wheel input in both GUI and TUI.

**Headless Toolkit Commands:**
- `Search(String)`: Triggers `ripgrep` workspace search.
- `Export(Format)`: Headless PDF/HTML generation.
- `GetStats(PathBuf)`: Word count and document metadata analysis.
- `ManageConfig(Action)`: Reads or modifies the `config.toml`.


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

The first tabs implementation keeps open-document ownership in `paperview-core::OpenDocuments`. Frontends can use the shared model to add, activate, and replace documents without duplicating path de-duplication rules.

The first LaTeX implementation keeps math semantics in `paperview-core` by
preserving inline math text and exposing display math as `Block::Math`.
Frontends render source-preserving math affordances until native formula
typesetting is selected.

The first Mermaid implementation keeps diagram semantics in `paperview-core` by
detecting `mermaid` fenced code blocks and exposing them as `Block::Diagram`.
Frontends render source-preserving diagram affordances until native diagram
rendering is selected.

The first table implementation keeps table structure in `paperview-core` by
consuming `pulldown-cmark` table events into `Block::Table`. Frontends render
from the shared alignments, header cells, and body rows instead of reparsing
Markdown text.

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
