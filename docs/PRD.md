# PaperView — Product Requirements Document (PRD)

## 1. Overview
**PaperView** is a native, cross-platform Markdown and technical document viewer written in Rust. It aims to provide a high-performance, distraction-free reading experience that feels native to every desktop environment.

### 1.1 Focus Areas
- **Beautiful Typography:** Prioritizing readability and visual clarity.
- **Fast Rendering:** Instantaneous document loading and 60 FPS scrolling.
- **Distraction-Free:** A clean UI that puts the content first.
- **Native Feel:** Utilizing system-native GUI primitives for performance and integration.
- **Technical Workflows:** Optimized for developers, researchers, and technical writers.

### 1.2 Out of Scope
PaperView is **not** intended to be:
- A productivity suite or Notion clone.
- A plugin-heavy workspace.
- A heavy Electron-based application.

**Vision:** "The *Preview.app* for Markdown and technical documents."

---

## 2. Vision & Principles
### 2.1 Core Principles
1. **Viewer-First:** Optimized primarily for reading, navigating, and understanding documents.
2. **Native Desktop Feel:** Lightweight, memory-efficient, and responsive. Avoids "web-app" sluggishness.
3. **Technical-Document Optimized:** Designed for Markdown, engineering notes, RFCs, and LaTeX/math content.

### 2.2 Design Philosophy
The application should feel **fast, quiet, focused, and native.**

---

## 3. Target Audience
| User Group | Primary Use Case |
| :--- | :--- |
| **Developers** | Project documentation, RFCs, architecture docs, READMEs. |
| **Researchers / Students** | Technical notes, scientific Markdown, math-heavy documents. |
| **Technical Writers** | Reviewing content with beautiful rendering and distraction-free UI. |

---

## 4. Platform Strategy
- **macOS:** Primary design target; must feel like a first-class citizen.
- **Linux:** First-class support for Wayland and Tiling WMs; optimized for developer workflows.
- **Windows:** Fully supported with a native look and feel.

---

## 5. Technology Stack
- **Language:** [Rust](https://www.rust-lang.org/)
- **GUI Framework:** [Iced](https://iced.rs/) (Rust-native, modern, cross-platform)
- **TUI Framework:** [Ratatui](https://ratatui.rs/) (The modern standard for Rust terminal UIs)
- **Markdown Parsing:** `pulldown-cmark`
- **File Watching:** `notify` (for live reload)
- **Serialization:** `serde` + `toml`

---

## 6. UI/UX Design (Reference)
*Detailed visual specs live in [docs/design/INDEX.md](design/INDEX.md)*

---

## 7. Functional Requirements

### 7.1 Markdown Support
- **Core:** Headings, paragraphs, bold/italic, links, lists, blockquotes, tables, horizontal rules.
- **Code Blocks:** Syntax highlighting, rounded containers, "Copy" button.
- **Images:** Responsive scaling, centered display, click-to-zoom (future).
- **Task Lists:** Interactive check-boxes.
- **LaTeX & Mermaid:** Native rendering within the MVP.

### 7.2 File Handling
- **Formats:** `.md`, `.markdown`, `.txt`.
- **Drag & Drop:** Supports dragging files into the window.
- **Recent Files:** Persistent history.
- **Live Reload:** Automatic refresh when the source file is saved (crucial for docs writers).

### 7.3 Navigation
- **Table of Contents:** Auto-generated from Markdown headers.
- **Scroll Sync:** Highlights current section in TOC during scrolling.
- **Search:** In-document text search.

### 7.4 CLI & Interaction Model
PaperView provides a context-aware CLI and a suite of "headless" tools.

**Primary Launch:**
- **`paperview <file>`**: Launches the **TUI** (Ratatui) version directly in the terminal.
- **`paperview -g` / `--gui <file>`**: Launches the **GUI** (Iced) version in a native window.
- **`paperview`**: Launches the TUI Dashboard (Recent Files).

**Subcommands (Documentation Toolkit):**
- **`paperview search <query>`**: Uses `ripgrep` to search the current workspace/folder and lists results in the TUI for selection.
- **`paperview export <file> --to [pdf|html]`**: Headless conversion of Markdown to "Paper" style documents.
- **`paperview config [path|edit]`**: Manages settings (TOML). `path` shows the file location; `edit` opens it in the default editor.
- **`paperview stats <file>`**: Prints metadata (Word count, Reading time, Heading structure) directly to the console without launching a UI.

**Keyboard & Mouse Support:** 
- Full keyboard navigation (`j/k`, `h/l`, `Tab`).
- **Mouse Support:** Click to switch tabs, click sidebar items to open, and use the scroll wheel to navigate documents.

---

## 8. Performance Goals
- **Startup Time:** < 500ms.
- **Memory Usage:** Significantly lower than Electron-based alternatives.
- **Smoothness:** Consistent 60 FPS scrolling even on large documents.

---

## 9. Roadmap
### Phase 1: MVP (v0.1)
- Native window with Iced.
- Basic Markdown rendering (LaTeX and Mermaid included).
- Left (History) and Right (TOC) sidebars.
- **Tabs:** Multiple documents open simultaneously.
- **Split View:** Side-by-side comparison of two documents.
- Dark/Light/Hybrid themes.
- Live reload and file opening.
- **Zen Mode:** A distraction-free UI state that hides sidebars and tabs.

### Phase 2: Enhanced Functionality
- **Editing Mode:** Toggle between Viewer and Editor.
    - Split-pane live preview.
    - Basic Markdown syntax highlighting in the editor.
    - Save functionality (overwriting the source file).
- **Global Search:** Ripgrep-powered search across the workspace.
- **Export:** Support for HTML and PDF export.

### Phase 3: Technical Advanced & Graph
- LaTeX/Math support ($inline$ and $$block$$).
- Full `.tex` support via Tectonic integration.
- **Knowledge Graph:** Visual connections between linked Markdown files.
- **Presentation Mode:** One-click conversion of headings/rules into slides.

### Phase 4: Long-term Explorations (Noted)
- **Git-Integrated Viewer:** See diffs and blame history inline.
- **WASM Extensions:** Plugin system for custom renderers.
- **TUI Version:** (In progress as part of the core workspace).

---

## 10. Inspiration
- **Design:** Preview.app, Typora, Zed, Bear, Linear.
- **Functionality:** Obsidian (Minimal mode), Raycast.
