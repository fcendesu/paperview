# GUI History Open

## Goal and Scope

Make GUI history entries clickable so users can reopen recent files from the left sidebar.

This plan covered:

- Add GUI messages and update handling.
- Convert history items into buttons.
- Open selected history paths through `Document::open`.
- Record and persist successfully opened documents.
- Show open failures in GUI status.
- Update docs.

Out of scope:

- Removing history entries.
- Multi-tab opening.
- Keyboard navigation in the GUI history rail.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/history.rs`
- `crates/paperview-gui/src/main.rs`
- `crates/paperview-gui/src/navigation.rs`
- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/history-sidebar.md`
- `docs/TASKS.md`

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
PAPERVIEW_HISTORY_PATH=<temp> cargo run -p paperview-gui -- docs/PRD.md
```

## Final Outcome

- GUI history rows now dispatch `OpenHistory` messages.
- Successful opens replace the active document and update persisted recent-file history.
- Failed opens surface in the GUI status line without changing the current document.
