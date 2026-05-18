# TUI Tab Close

## Goal

Add a first Ratatui workflow for closing open document tabs.

## Scope

- `crates/paperview-tui/src/app.rs`
- README, task tracker, and tabs feature spec

## Implementation Steps

1. Add an `x` keybinding to close the active TUI tab.
2. Use `paperview_core::OpenDocuments::close` for active-index fallback.
3. Reload reader lines, TOC, search results, and watcher state around the new
   active tab after close.
4. Exit the reader cleanly when the final tab closes.
5. Add focused tests for neighbor activation and final-tab exit.
6. Update docs and trackers.

## Outcome

The TUI now supports closing the active tab with `x`. Closing a tab activates
the next available neighbor using the shared tab model, and closing the final
tab exits the reader.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-tui tab
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
