# TUI Task Toggle Writeback

## Goal

Let terminal users toggle Markdown task checkboxes without leaving PaperView.

## Completed

- Added `Space` key handling for toggling the task checkbox at the current
  reader line.
- Reused `paperview-core::toggle_task_line_source` for Markdown writeback.
- Scoped writeback to file-backed active documents.
- Reloaded the active document after a successful toggle.
- Added tests for successful file-backed toggles and non-file/no-task no-op
  paths.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core task`
- `cargo test -p paperview-gui task`
- `cargo test -p paperview-tui task`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
