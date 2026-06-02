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
- `paperview-tui`: a Ratatui terminal reader with hybrid theme styling,
  recent-file dashboard, tabs, table of contents, active-section highlighting,
  scrolling, and live reload.

Supported input formats:

- `.md`
- `.markdown`
- `.txt`

Implemented Markdown rendering is still intentionally basic: headings,
paragraphs, lists, blockquotes, and table cells with basic inline styling, code
blocks, tables, task-list markers, rules, LaTeX display math panels with
readable previews, Mermaid diagram panels with simple flowchart previews, and
image metadata panels are supported. Richer technical-document features such as
full LaTeX typesetting, full Mermaid rendering, advanced Split View controls,
and richer PDF layout are still on the roadmap.

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
- Header search field with previous/next match navigation and highlighted
  rendered matches.
- Heading, paragraph, list, blockquote, and table-cell bold, italic, inline code,
  and clickable link styling.
- Bordered Markdown tables with shaded headers.
- Clickable task-list markers with Markdown writeback for file-backed documents.
- Standalone local and remote bitmap image previews with metadata fallback.
- Source-preserving display math panels with readable previews.
- Mermaid diagram panels with simple native flowchart previews.
- Multiple document tabs with close controls.
- Native drag-and-drop, including multiple dropped files into tabs.
- Split View for comparing the active tab with another open tab, including a
  header toggle, secondary tab selection, keyboard resizing, and divider drag
  resizing.
- Live reload when the active file changes on disk.
- Zen Mode with `Cmd + Shift + F` on macOS or `Ctrl + Shift + F` elsewhere.
- Split View toggle with `Cmd + \` on macOS or `Ctrl + \` elsewhere.
- Split View resize with `Cmd + [` / `Cmd + ]` on macOS or `Ctrl + [` /
  `Ctrl + ]` elsewhere.
- Drag the Split View divider to resize panes directly.

## Run The TUI

Open a file directly:

```sh
cargo run -p paperview-tui -- docs/PRD.md
```

Open multiple files as TUI tabs:

```sh
cargo run -p paperview-tui -- docs/PRD.md README.md
```

Open the recent-files dashboard:

```sh
cargo run -p paperview-tui
```

Print document stats without launching the TUI:

```sh
cargo run -p paperview-tui -- stats docs/PRD.md
cargo run -p paperview-tui -- stats docs/PRD.md --json
```

Print a local load/parse/render performance baseline:

```sh
cargo run -p paperview-tui -- perf docs/PRD.md
```

Show or open the config file:

```sh
cargo run -p paperview-tui -- config path
cargo run -p paperview-tui -- config edit
```

Search a workspace or folder:

```sh
cargo run -p paperview-tui -- search PaperView docs
cargo run -p paperview-tui -- search PaperView docs --interactive
```

Export a document to HTML without launching the TUI:

```sh
cargo run -p paperview-tui -- export docs/PRD.md --to html
```

Use `--to pdf` to write a basic text-first PDF beside the source document:

```sh
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
```

Compile a `.tex` entry file through Tectonic without launching the TUI:

```sh
cargo run -p paperview-tui -- tex compile resume.tex
```

This requires a `tectonic` executable on `PATH` and writes the generated PDF
beside the source file.

TUI controls:

- `j` / `Down`: scroll or move selection down.
- `k` / `Up`: scroll or move selection up.
- `g`: jump to top in the reader.
- `G`: jump to bottom in the reader.
- `e`: enter Editing Mode for the active document.
- `p`: enter Presentation Mode for the active document.
- `Space`: toggle a task checkbox at the current reader line for file-backed documents.
- `[` / `]`: switch to the previous or next tab.
- `\`: toggle Split View when multiple tabs are open.
- `<` / `>`: resize the Split View primary pane.
- `{` / `}`: switch Split View's secondary pane.
- `z`: toggle Zen Mode.
- `x`: close the active tab.
- `Tab`: switch focus between the reader and table of contents.
- `Enter`: jump to the selected TOC heading when the TOC is focused.
- `/`: search within the current document.
- `n` / `N`: jump to the next or previous search match.
- In the dashboard, `Enter`: open the selected recent file.
- In interactive workspace search, `Enter`: open the selected match near its line.
- In Presentation Mode, `Space` / `Right` / `n`: next slide; `Left` / `b`:
  previous slide; `Home` / `End`: first or last slide; `Esc` / `q`: return to
  the reader.
- In Editing Mode, arrow keys move the cursor, `Home` / `End` jump within the
  current line, `PageUp` / `PageDown` move by larger line chunks, `Backspace` /
  `Delete` remove text, `Ctrl+S` saves edits, `Ctrl+P` toggles the preview
  pane, `Ctrl+Up` / `Ctrl+Down` and `Ctrl+PageUp` / `Ctrl+PageDown` scroll the
  preview, and `Esc` returns to the reader. Dirty edits require a second
  discard action before `Esc`, tab switch, or tab close drops unsaved changes.
- `q` / `Esc`: quit the current TUI view.

The TUI highlights the active table-of-contents section while you scroll, can
jump through headings from the TOC, can switch between open document tabs, can
compare two tabs in Split View, toggle a focused Zen Mode, and close the active
tab. It can enter a slide-focused Presentation Mode generated from Markdown
rules or top-level headings. It preserves LaTeX display math with readable previews plus Mermaid
diagram source with simple flowchart previews. Markdown tables render as aligned plain text, and
standalone images render as Markdown image text. Heading, paragraph, list,
blockquote, and table-cell inline styling renders in Markdown-shaped text.
Task-list markers render as `- [x]` and `- [ ]` lines. The TUI also supports
file-backed task checkbox toggles, case-insensitive in-document search with
match navigation, highlighted match lines, and a cursor-addressable Editing
Mode source buffer with live rendered preview for file-backed documents.

The GUI can also enter Presentation Mode from the header `Present` button or
`Cmd/Ctrl+P`, renders slides through the normal reader, and provides header
previous/next controls plus `View` to return to reading. While presenting,
`Space`, `Right`, or `n` advances; `Left` or `b` goes back; `Home` and `End`
jump to the first or last slide; `Esc` exits.

The headless stats command prints word, line, character, reading-time, and
heading-structure metadata for a document, with `--json` for automation.
The headless perf command prints document size, parse shape, rendered TUI line
count, deterministic memory estimate, memory/load target status, and
read/parse/render timings for local baseline checks.
The config commands print or open PaperView's TOML config file, creating a
default file before edit when needed. The GUI and TUI currently persist Zen
Mode, Split View width, and the current `hybrid` theme preference in that
config file.
The workspace search command prints ripgrep-backed `path:line:column` results
without launching the TUI, or opens a Ratatui result picker with `--interactive`.
The GUI left rail also includes workspace search; submit a query with `Find` or
Enter, then click a result to open the matched file near its source line.
The export command writes standalone PaperView-styled HTML with heading anchors
or a text-first PDF with basic layout beside the source document and prints the
generated path.

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
- Full LaTeX typesetting and full Mermaid rendering.
- Richer PDF layout and embedded image rendering.
- Richer Markdown rendering for nested task-list structure.
- Performance measurement against startup, scrolling, and memory targets.

See [`docs/TASKS.md`](docs/TASKS.md) for the current implementation tracker.
