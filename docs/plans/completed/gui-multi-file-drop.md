# GUI Multi-File Drop

## Goal and Scope

Make multi-file drag-and-drop semantics explicit for the GUI tab model.

This plan covered:

- Route dropped files through a shared multi-path update path.
- Open every supported dropped file as a tab.
- Preserve existing path de-duplication and activation behavior.
- Keep unsupported dropped files from preventing supported files in the same drop batch from opening.
- Update drag/drop, tabs, and tracker docs.

Out of scope:

- Folder drops.
- Directory expansion.
- Drag/drop in the TUI.
- Visual previews of all hovered files.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `docs/features/drag-and-drop.md`
- `docs/features/tabs.md`
- `docs/TASKS.md`

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Smoke test:

```sh
cargo run -p paperview-gui
```

Then drag multiple supported documents into the window and confirm each opens as a tab.

## Final Outcome

- Runtime file-drop events and tests use the same `OpenDroppedFiles` update path.
- Supported dropped files open as tabs.
- Unsupported files report an error without removing successfully opened tabs from the batch.
