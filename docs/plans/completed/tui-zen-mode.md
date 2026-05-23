# TUI Zen Mode

## Goal

Add a focused terminal reading layout that keeps the active document visible
while hiding secondary panes.

## Completed

- Added `z` key handling for toggling TUI Zen Mode.
- Hid the tab line, table-of-contents pane, and split side pane while Zen Mode
  is enabled.
- Kept the header, active reader, scroll, search, tab state, and Split View
  state intact.
- Forced focus back to the reader while Zen Mode is active.
- Added focused tests for Zen focus behavior and header rendering.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui zen`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
