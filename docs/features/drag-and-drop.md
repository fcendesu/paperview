# Drag And Drop

## Product Behavior

PaperView lets GUI users open supported documents by dragging a file onto the window.

Current behavior:

- Shows a subtle accent border while a file is hovering over the GUI window.
- Shows a header status prompt with the hovered path.
- Opens the dropped file through the same document loader used by launch arguments and history clicks.
- Opens multiple dropped files as tabs when the platform emits multiple dropped-file events.
- Records and persists successfully opened dropped files in recent-file history.
- Shows unsupported-file and read failures in the GUI status line while keeping supported files from the same drop batch open.

Multiple dropped files are handled as individual window events by the platform; PaperView routes them through the same multi-path drop handler used by tests and future platform batching.

## Implementation Notes

- GUI event subscription wiring lives in `crates/paperview-gui/src/app.rs`.
- `iced::event::listen_with` filters `window::Event::FileHovered`, `FilesHoveredLeft`, and `FileDropped`.
- Native `FileDropped` events map to `Message::OpenDroppedFiles` with the dropped path.
- `Message::OpenDroppedFiles` opens every supported path and reports the last unsupported/read failure, if any.
- Supported file validation remains centralized in `paperview-core::SupportedFileType` through `Document::open`.
- Hover styling lives in `crates/paperview-gui/src/theme.rs` as a shell accent border.

## Open Decisions

- Folder drops are deferred until workspace search or project opening exists.
- TUI does not have drag-and-drop support.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For visual smoke testing:

```sh
cargo run -p paperview-gui
```

Then drag a supported `.md`, `.markdown`, or `.txt` file into the window and confirm it opens.
