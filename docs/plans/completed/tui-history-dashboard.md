# TUI History Dashboard

## Goal and Scope

Make `paperview-tui` without a file launch a Ratatui recent-files dashboard.

This plan covers:

- Load persisted history from `HistoryStore`.
- Show recent files in a selectable list.
- Support `q`/Esc quit, `j`/Down and `k`/Up selection, and Enter to open the selected file.
- Reuse the existing reader shell after opening a selected file.
- Update docs and tracker.

Out of scope:

- Deleting history entries.
- Grouping by date.
- Search/filter.
- Mouse support.

## Affected Paths

- `crates/paperview-tui/src/main.rs`
- `crates/paperview-tui/src/app.rs`
- `docs/features/history-sidebar.md`
- `docs/TASKS.md`

## Implementation Steps

1. Add a dashboard entrypoint to the TUI app.
2. Render persisted history with Ratatui list widgets.
3. Implement selection and open-selected behavior.
4. Update docs and tracker.
5. Run required checks and PTY smoke tests.

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
PAPERVIEW_HISTORY_PATH=<temp> cargo run -p paperview-tui
```

## Progress Notes

- Started after the initial Ratatui reader shell landed.
- Added no-file dashboard that loads persisted history, supports bounded selection, and opens selected files in the reader shell.
- Verified formatting, Clippy, workspace tests, an empty-dashboard PTY smoke, and a seeded-history PTY smoke that opened `docs/PRD.md`.
