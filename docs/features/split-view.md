# Split View

## Product Behavior

Split View lets the GUI compare two open documents side by side. The active tab stays
the primary reader; the secondary pane uses another already-open tab.

Current GUI behavior:

- Toggle Split View from the header button.
- Toggle Split View with the platform command shortcut: `Cmd + \` on macOS and
  `Ctrl + \` elsewhere.
- Split View only activates when at least two tabs are open.
- When enabled, the active document renders in the left pane and the secondary
  document renders in the right pane.
- Choose the secondary pane from any non-active tab while Split View is on.
- Selecting the document currently used as the secondary pane retargets the
  secondary pane to another open tab when one exists.
- Closing the secondary tab retargets the split pane or disables Split View when
  no secondary tab remains.
- Zen Mode takes precedence and shows only the active reader.

## Implementation Notes

- `paperview-gui` stores the secondary pane as an optional open-document index.
- `OpenDocuments` remains the shared tab/document model; no separate core split
  model exists yet.
- The header owns the global Split View toggle.
- Non-active tabs show a compact secondary-pane selector while Split View is on.
- History and table-of-contents sidebars remain visible in Split View.
- The table of contents follows the active document only.
- Live reload remains scoped to the active document watcher.

## Open Decisions

- Independent scroll persistence and scroll synchronization are deferred.
- Drag-to-resize split panes are deferred.
- TUI Split View is deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI smoke test with two tabs open and Split View toggled.
