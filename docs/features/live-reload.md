# Live Reload

## Product Behavior

PaperView refreshes the active GUI or TUI document when its source file changes on disk. This supports the writer workflow where Markdown is edited in another app while PaperView remains open as a reader.

Current behavior:

- Watches the active GUI and TUI document path when a file is open.
- Reloads the current document after a relevant create, modify, rename, metadata, or remove event touches that path.
- Updates the active title, parsed document, and table of contents after a successful reload.
- Refreshes the GUI recent-file entry when the GUI reloads successfully.
- Shows reload failures in the GUI status line or TUI status area.
- Stops watching when there is no active document.

Dashboard watching, multi-tab watching, split-pane watching, and exact scroll-position restoration are deferred.

## Implementation Notes

- Core watcher logic lives in `crates/paperview-core/src/watcher.rs`.
- `paperview-core` owns the `notify` dependency and exposes `watch_file` plus `WatchEvent`.
- The watcher observes the parent directory non-recursively and filters events to the active path. This catches common editor save strategies that replace a file instead of writing it in place.
- GUI subscription wiring lives in `crates/paperview-gui/src/app.rs`.
- `Message::FileChanged` reloads only when the changed path still matches the active document path.
- TUI watcher wiring lives in `crates/paperview-tui/src/app.rs`.
- The TUI reader uses short input polling ticks so watcher events can be handled without losing keyboard responsiveness.
- TUI reload preserves the current scroll offset within the new document bounds.

## Open Decisions

- Debouncing duplicate editor events is deferred until repeated reloads become visible or measurable.
- Scroll-position preservation needs reader scroll state before it can be implemented exactly.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For smoke testing:

```sh
PAPERVIEW_HISTORY_PATH=<temp-history> cargo run -p paperview-gui -- <temp-doc>
cargo run -p paperview-tui -- <temp-doc>
```

Edit `<temp-doc>` externally and confirm the rendered title/body refreshes.
