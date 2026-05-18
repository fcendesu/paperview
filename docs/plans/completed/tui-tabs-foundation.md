# TUI Tabs Foundation

## Goal

Bring the Ratatui reader onto the shared open-document tab model and provide a
first keyboard-driven tab workflow.

## Scope

- `crates/paperview-tui/src/main.rs`
- `crates/paperview-tui/src/app.rs`
- README, task tracker, and tabs feature spec

## Implementation Steps

1. Let `paperview-tui [file ...]` open multiple documents.
2. Replace the TUI reader's single-document field with
   `paperview_core::OpenDocuments`.
3. Render a compact tab row in the TUI header.
4. Add `[` and `]` navigation for previous and next tab selection.
5. Reload reader lines, TOC, search results, and file watcher state when the
   active tab changes.
6. Add focused tests for tab navigation, tab rendering, and multi-file command
   loading.

## Outcome

The TUI can now open multiple files at launch and switch between them with
`[` and `]`. The active tab drives the reader content, TOC, search result set,
header title, and live-reload target. TUI tab closing and reordering are
deferred.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-tui tab
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
