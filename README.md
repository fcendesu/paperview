# PaperView

PaperView is a native Markdown and technical-document viewer written in Rust.
The goal is a fast, quiet, viewer-first reading experience for project docs,
RFCs, research notes, and technical writing.

PaperView is intentionally not an Electron-style workspace or a productivity
suite. Think "Preview.app for Markdown": open a document, read it comfortably,
navigate quickly, and keep it in sync while editing elsewhere.

## Current Status

PaperView is in early MVP development. The workspace currently contains:

- `paperview-core`: document loading, Markdown parsing, recent-file history,
  live-reload watching, in-document search, and shared open-document tab state.
- `paperview-gui`: an Iced desktop reader with history, table of contents,
  tabs, drag-and-drop, live reload, and Zen Mode.
- `paperview-tui`: a Ratatui terminal reader with recent-file dashboard,
  table of contents, active-section highlighting, scrolling, and live reload.

Supported input formats:

- `.md`
- `.markdown`
- `.txt`

Implemented Markdown rendering is still intentionally basic: headings,
paragraphs, lists, blockquotes, and table cells with basic inline styling, code
blocks, tables, task-list markers, rules, LaTeX display math panels with
readable previews, Mermaid diagram panels with simple flowchart previews, and
image metadata panels are supported. Richer technical-document features such as
full LaTeX typesetting, full Mermaid rendering, remote image fetching,
interactive task toggles, search, advanced Split View controls, and export are
still on the roadmap.

## Run The GUI

```sh
cargo run -p paperview-gui -- docs/PRD.md
```

GUI highlights:

- History sidebar with persisted recent files.
- Click history entries to reopen documents.
- Table-of-contents sidebar generated from headings.
- Active table-of-contents highlighting while scrolling the GUI reader.
- Click TOC entries to jump the active GUI reader.
- Header search field with previous/next match navigation.
- Heading, paragraph, list, blockquote, and table-cell bold, italic, inline code,
  and clickable link styling.
- Bordered Markdown tables with shaded headers.
- Read-only checked and unchecked task-list markers.
- Standalone local bitmap image previews with metadata fallback.
- Source-preserving display math panels with readable previews.
- Mermaid diagram panels with simple native flowchart previews.
- Multiple document tabs with close controls.
- Native drag-and-drop, including multiple dropped files into tabs.
- Split View for comparing the active tab with another open tab, including a
  header toggle, secondary tab selection, and keyboard resizing.
- Live reload when the active file changes on disk.
- Zen Mode with `Cmd + Shift + F` on macOS or `Ctrl + Shift + F` elsewhere.
- Split View toggle with `Cmd + \` on macOS or `Ctrl + \` elsewhere.
- Split View resize with `Cmd + [` / `Cmd + ]` on macOS or `Ctrl + [` /
  `Ctrl + ]` elsewhere.

## Run The TUI

Open a file directly:

```sh
cargo run -p paperview-tui -- docs/PRD.md
```

Open the recent-files dashboard:

```sh
cargo run -p paperview-tui
```

TUI controls:

- `j` / `Down`: scroll or move selection down.
- `k` / `Up`: scroll or move selection up.
- `g`: jump to top in the reader.
- `G`: jump to bottom in the reader.
- `Tab`: switch focus between the reader and table of contents.
- `Enter`: jump to the selected TOC heading when the TOC is focused.
- `/`: search within the current document.
- `n` / `N`: jump to the next or previous search match.
- In the dashboard, `Enter`: open the selected recent file.
- `q` / `Esc`: quit the current TUI view.

The TUI highlights the active table-of-contents section while you scroll, can
jump through headings from the TOC, and preserves LaTeX display math plus
Mermaid diagram source. Markdown tables render as aligned plain text, and
standalone images render as Markdown image text. Heading, paragraph, list,
blockquote, and table-cell inline styling renders in Markdown-shaped text.
Task-list markers render as `- [x]` and `- [ ]` lines. The TUI also supports
case-insensitive in-document search with match navigation.

## Development Checks

Run these before finishing code changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

Run tests when touching behavior:

```sh
cargo test --workspace
```

The canonical verification guide lives in
[`docs/quality/CHECKS.md`](docs/quality/CHECKS.md).

## Repository Map

```text
crates/
  paperview-core/  shared document, parser, history, watcher, and tab logic
  paperview-gui/   Iced desktop frontend
  paperview-tui/   Ratatui terminal frontend
docs/
  PRD.md           product vision and roadmap
  TASKS.md         implementation tracker
  arch/            architecture notes
  design/          visual and interaction design notes
  features/        feature specs and implementation records
  plans/           active and completed execution plans
  quality/         verification expectations
```

For project decisions, prefer the documents under `docs/`; they are the
repository knowledge base and source of truth.

## Roadmap Snapshot

Near-term MVP work includes:

- Exact scroll geometry from rendered layout rectangles.
- Split View drag resizing.
- Full LaTeX typesetting and full Mermaid rendering.
- Documentation-toolkit commands.
- Richer Markdown rendering for remote images, exported anchors, and interactive
  task toggles.
- Performance measurement against startup, scrolling, and memory targets.

See [`docs/TASKS.md`](docs/TASKS.md) for the current implementation tracker.
