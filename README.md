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
  live-reload watching, and shared open-document tab state.
- `paperview-gui`: an Iced desktop reader with history, table of contents,
  tabs, drag-and-drop, live reload, and Zen Mode.
- `paperview-tui`: a Ratatui terminal reader with recent-file dashboard,
  table of contents, scrolling, and live reload.

Supported input formats:

- `.md`
- `.markdown`
- `.txt`

Implemented Markdown rendering is still intentionally basic: headings,
paragraphs, lists, blockquotes, code blocks, and rules are supported. Richer
technical-document features such as LaTeX, Mermaid, images, tables, task lists,
search, advanced Split View controls, and export are still on the roadmap.

## Run The GUI

```sh
cargo run -p paperview-gui -- docs/PRD.md
```

GUI highlights:

- History sidebar with persisted recent files.
- Click history entries to reopen documents.
- Table-of-contents sidebar generated from headings.
- Multiple document tabs with close controls.
- Native drag-and-drop, including multiple dropped files into tabs.
- Split View for comparing the active tab with another open tab.
- Live reload when the active file changes on disk.
- Zen Mode with `Cmd + Shift + F` on macOS or `Ctrl + Shift + F` elsewhere.
- Split View toggle with `Cmd + \` on macOS or `Ctrl + \` elsewhere.

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
- `Enter`: open the selected recent file from the dashboard.
- `q` / `Esc`: quit the current TUI view.

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

- Scroll synchronization.
- Split View resizing, secondary selection, and scroll synchronization.
- LaTeX and Mermaid support.
- Search and documentation-toolkit commands.
- Richer Markdown rendering for tables, images, links, and task lists.
- Performance measurement against startup, scrolling, and memory targets.

See [`docs/TASKS.md`](docs/TASKS.md) for the current implementation tracker.
