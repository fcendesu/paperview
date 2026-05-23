# TUI Hybrid Theme

## Goal

Map PaperView's dark-shell and cream-reader visual language into the Ratatui
frontend.

## Completed

- Added `crates/paperview-tui/src/theme.rs` with named TUI style helpers.
- Applied shell, reader, tab, TOC, history, and search highlight styles through
  the TUI app and render modules.
- Kept visual styling out of core logic.
- Updated feature, design, README, and task-tracker documentation.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui theme`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
