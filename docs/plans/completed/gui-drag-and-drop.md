# GUI Drag And Drop

## Goal and Scope

Let users open supported documents by dropping files into the GUI window.

This plan covered:

- Subscribe to Iced file hover and drop window events.
- Show a subtle hover affordance while a file is over the window.
- Route dropped paths through the existing GUI document opener.
- Record and persist successful dropped-file opens.
- Reuse existing unsupported-file and read-error status handling.
- Update feature, design, and tracker docs.

Out of scope:

- Multi-file drop queues.
- Folder drops.
- Custom per-extension drop rejection before `Document::open`.
- TUI-specific drag and drop.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/drag-and-drop.md`
- `docs/features/file-opening.md`
- `docs/features/INDEX.md`
- `docs/design/INDEX.md`
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

Then drag a supported Markdown or text file into the window and confirm it opens.

## Final Outcome

- GUI subscribes to native file hover/drop events.
- Hovering files show a shell accent border and header prompt.
- Dropped files open through the shared GUI opener, preserving history and live reload behavior.
