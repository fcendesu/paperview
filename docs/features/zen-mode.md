# Zen Mode

## Product Behavior

PaperView has a focused reading mode that hides secondary chrome around the
active document.

Current GUI behavior:

- Toggles from Cmd+Shift+F on macOS and Ctrl+Shift+F elsewhere.
- Hides the tab bar, History sidebar, and table-of-contents sidebar.
- Keeps the header visible so document status and file errors remain available.
- Loads the initial Zen Mode preference from config and saves changes when toggled.
- Keeps the active reader, live reload, history persistence, and drag-and-drop file opening behavior intact.
- Toggling again restores the full shell layout.

Current TUI behavior:

- Toggles with `z`.
- Hides the tab line, table-of-contents pane, and split side pane.
- Keeps the header visible so document status and file errors remain available.
- Keeps the active reader, scrolling, search, tab state, and Split View state
  intact.
- Forces focus back to the reader while Zen Mode is on.
- Loads the initial Zen Mode preference from config and saves changes when toggled.
- Toggling again restores the normal TUI shell layout.

Zen Mode requires an active document to feel useful, but the toggle state is allowed even when no document is open so later file opens can render in the focused layout.

## Implementation Notes

- GUI layout state lives in `crates/paperview-gui/src/app.rs` as `is_zen`.
- TUI layout state lives in `crates/paperview-tui/src/app.rs` as `is_zen`.
- `Message::ToggleZen` flips the layout state.
- The GUI and TUI readers persist `zen_mode` through `paperview-core::ConfigStore`.
- The GUI runtime event subscription maps the platform command shortcut to `ToggleZen`.
- The normal layout renders History, reader, and Navigation sidebars; the Zen layout renders only header and reader.
- The normal TUI layout renders header, reader, optional split pane, and TOC;
  the Zen layout renders only header and active reader.

## Open Decisions

- Header auto-hide is deferred until there is reader focus or scroll-state tracking.
- A visible toolbar toggle is deferred until the header controls are designed.
- Platform-specific shortcut discoverability is deferred until the header controls are designed.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For visual smoke testing:

```sh
cargo run -p paperview-gui -- docs/PRD.md
```

Press Cmd/Ctrl+Shift+F and confirm the sidebars and tab bar toggle away and back.
