# History Persistence

## Goal and Scope

Persist recent files so the GUI History sidebar survives app restarts.

This plan covers:

- TOML serialization for core `History` and `FileEntry`.
- A path-based `HistoryStore` with load/save behavior.
- A default app-data history path for GUI startup.
- GUI bootstrap that loads, records, and saves recent files.
- Documentation and tracker updates.

Out of scope:

- Click-to-open history items.
- Date grouping and timestamps.
- User-facing config commands.
- TUI history UI.

## Affected Paths

- `crates/paperview-core/src/history.rs`
- `crates/paperview-core/Cargo.toml`
- `crates/paperview-gui/src/app.rs`
- `docs/features/history-sidebar.md`
- `docs/TASKS.md`

## Implementation Steps

1. Add serde/toml support to core.
2. Implement `HistoryStore` load/save with missing-file fallback.
3. Add default platform-aware storage path.
4. Wire GUI startup to persisted history.
5. Run required checks and smoke-test GUI launch.

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p paperview-gui -- docs/PRD.md
```

## Progress Notes

- Started after the non-persistent history sidebar foundation landed.
- Added TOML-backed `HistoryStore` with missing-file fallback and directory creation on save.
- Wired GUI startup to load persisted history, record opened documents, and save updates.
- Verified formatting, Clippy, workspace tests, and a GUI smoke launch using `PAPERVIEW_HISTORY_PATH`.
