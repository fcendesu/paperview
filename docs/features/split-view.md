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
- Loads the initial primary pane width from config and saves changes when resized by keyboard or drag.
- Split View only activates when at least two tabs are open.
- When enabled, the active document renders in the left pane and the secondary
  document renders in the right pane.
- The secondary pane tracks the primary pane's normalized scroll progress while
  Split View is enabled.
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
- Resize the primary pane with `<` / `>` while Split View is on.
- Loads the initial primary pane width from config and saves changes when resized.
- The left active pane owns scrolling, search, and TOC highlighting.
- The right side pane tracks the active pane's relative scroll progress.
- Selecting the document currently used as the secondary pane retargets the
  secondary pane to another open tab when one exists.
- Closing the secondary tab retargets the split pane or disables Split View when
  no secondary tab remains.

## Implementation Notes

- `paperview-core::SplitViewState` stores the secondary pane index, primary
  width, enable/disable state, retargeting behavior, side-pane cycling, and
  bounded resize rules.
- `paperview-core::SplitResize` represents grow/shrink resize commands shared
  by GUI and TUI.
- `OpenDocuments` remains the shared tab/document model while `SplitViewState`
  owns the split-specific relationship between the active and secondary tabs.
- `paperview-tui` renders a cached line buffer for the side pane.
- TUI side-pane cycling uses the shared non-active-tab cycling behavior.
- The primary pane width is stored as a bounded 30/70 to 70/30 ratio; it
  defaults to 50/50 and is persisted through `paperview-core::ConfigStore`.
- The header owns the global Split View toggle.
- Non-active tabs show a compact secondary-pane selector while Split View is on.
- The draggable divider updates the same bounded ratio used by keyboard
  resizing.
- History and table-of-contents sidebars remain visible in Split View.
- The table of contents follows the active document only.
- Live reload remains scoped to the active document watcher.
- Split scroll synchronization maps primary scroll progress to the secondary
  line buffer with `paperview-core::synced_scroll_offset`.

## Open Decisions

- Independent scroll persistence is deferred.
- Per-document Split View ratio persistence is deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI smoke test with two tabs open and Split View toggled.
- TUI smoke test with two files open and Split View toggled.
