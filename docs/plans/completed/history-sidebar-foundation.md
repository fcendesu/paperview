# History Sidebar Foundation

## Goal and Scope

Add the first shared recent-file model and show it in the GUI's left History sidebar.

This plan covers:

- Core `FileEntry` metadata derived from loaded documents.
- Core `History` ordering and path de-duplication behavior.
- GUI left sidebar scaffold that shows the current document as a history item.
- Documentation and tracker updates.

Out of scope:

- Persistence to disk.
- Clicking history entries to reopen files.
- Grouping entries by date.
- TUI history UI.

## Affected Paths

- `crates/paperview-core/src/history.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/history.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/history-sidebar.md`
- `docs/TASKS.md`

## Implementation Steps

1. Add core `FileEntry` and `History`.
2. Build initial GUI state history from the loaded document.
3. Add a static left History sidebar to the loaded-document layout.
4. Update docs and tracker.
5. Run required checks and smoke-test GUI launch.

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p paperview-gui -- docs/PRD.md
```

## Progress Notes

- Started after TOC parity was completed.
- Added core in-memory history metadata and GUI left sidebar rendering.
- Verified formatting, Clippy, workspace tests, and a short GUI launch smoke with `docs/PRD.md`.
