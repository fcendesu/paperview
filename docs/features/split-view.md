# Split View

## Product Behavior

Split View compares two open documents side by side. The active tab stays the
primary reader; the secondary pane uses another already-open tab.

Current GUI behavior:

- Toggle Split View from the header button.
- Toggle Split View with the platform command shortcut: `Cmd + \` on macOS and
  `Ctrl + \` elsewhere.
- Resize the primary pane with `Cmd + ]` / `Cmd + [` on macOS and `Ctrl + ]` /
  `Ctrl + [` elsewhere.
- Resize the split by dragging the vertical divider between panes.
- Split View only activates when at least two tabs are open.
- When enabled, the active document renders in the left pane and the secondary
  document renders in the right pane.
- Choose the secondary pane from any non-active tab while Split View is on.
- Selecting the document currently used as the secondary pane retargets the
  secondary pane to another open tab when one exists.
- Closing the secondary tab retargets the split pane or disables Split View when
  no secondary tab remains.
- Zen Mode takes precedence and shows only the active reader.

Current TUI behavior:

- Toggle Split View with `\`.
- Split View only activates when at least two tabs are open.
- When enabled, the active document renders in the left pane and the secondary
  open tab renders in the right pane.
- Cycle the secondary pane with `{` / `}` while Split View is on.
- The left active pane owns scrolling, search, and TOC highlighting.
- Selecting the document currently used as the secondary pane retargets the
  secondary pane to another open tab when one exists.
- Closing the secondary tab retargets the split pane or disables Split View when
  no secondary tab remains.

## Implementation Notes

- `paperview-gui` stores the secondary pane as an optional open-document index.
- `paperview-tui` stores the secondary pane as an optional open-document index
  and renders a cached line buffer for the side pane.
- TUI side-pane cycling walks non-active tabs only and wraps at either end.
- `OpenDocuments` remains the shared tab/document model; no separate core split
  model exists yet.
- The header owns the global Split View toggle.
- Non-active tabs show a compact secondary-pane selector while Split View is on.
- The primary pane width is stored as a bounded 30/70 to 70/30 ratio in GUI
  state; it defaults to 50/50 and is not persisted yet.
- The draggable divider updates the same bounded ratio used by keyboard
  resizing.
- History and table-of-contents sidebars remain visible in Split View.
- The table of contents follows the active document only.
- Live reload remains scoped to the active document watcher.

## Open Decisions

- Independent scroll persistence and scroll synchronization are deferred.
- Split ratio persistence is deferred.
- TUI split resizing is deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI smoke test with two tabs open and Split View toggled.
- TUI smoke test with two files open and Split View toggled.
