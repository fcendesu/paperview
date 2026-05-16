# Drag And Drop

## Product Behavior

PaperView lets GUI users open supported documents by dragging a file onto the window.

Current behavior:

- Shows a subtle accent border while a file is hovering over the GUI window.
- Shows a header status prompt with the hovered path.
- Opens the dropped file through the same document loader used by launch arguments and history clicks.
- Records and persists successfully opened dropped files in recent-file history.
- Shows unsupported-file and read failures in the GUI status line.

Multiple dropped files are handled as individual window events by the platform, but PaperView currently treats each event as a direct open request and ends on the last successfully opened file.

## Implementation Notes

- GUI event subscription wiring lives in `crates/paperview-gui/src/app.rs`.
- `iced::event::listen_with` filters `window::Event::FileHovered`, `FilesHoveredLeft`, and `FileDropped`.
- `Message::FileDropped` clears the drag-hover state and routes to `open_path`.
- Supported file validation remains centralized in `paperview-core::SupportedFileType` through `Document::open`.
- Hover styling lives in `crates/paperview-gui/src/theme.rs` as a shell accent border.

## Open Decisions

- Multi-file drop queues are deferred until tabs exist.
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
