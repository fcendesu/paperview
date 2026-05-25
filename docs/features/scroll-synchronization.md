# Scroll Synchronization

## Product Behavior

Scroll Synchronization keeps navigation context aligned with the document reader.

Current GUI behavior:

- The active reader reports scroll progress through Iced viewport events.
- The right-hand table of contents highlights the section matching the active
  reader's current scroll position.
- Clicking a table-of-contents entry jumps the active reader to that section.
- The first heading is highlighted when a document is opened, selected, reloaded,
  or when the reader is at the top.
- In Split View, the table of contents follows the active/primary pane only.
- In Split View, the secondary pane follows the active/primary pane's normalized
  scroll progress.

Current TUI behavior:

- The terminal reader tracks rendered line anchors for document blocks.
- The right-hand table of contents marks the active section as the user scrolls.
- `Tab` toggles focus between reader scrolling and TOC selection.
- While the TOC is focused, `j` / `k` or arrow keys move the TOC selection and
  `Enter` jumps the reader to the selected heading.
- The active TOC marker is preserved across live reload after scroll clamping.
- In Split View, the side pane follows the active pane's relative rendered-line
  scroll progress.

## Implementation Notes

- `paperview-gui` stores the active TOC item as a heading block index.
- The current mapping uses reader-aware heading anchors estimated from block
  type, typography, wrapping, and spacing. This is closer to visible reader
  geometry than raw block counts, but exact Iced layout capture is still
  deferred.
- `paperview-core` already exposes heading block indices through `TocItem`, so
  no new core data model was needed for the first GUI slice.
- `reader::view_with_scroll` adapts Iced scrollable viewport changes into a
  normalized scroll-progress message for the app state.
- `navigation::view` receives the active block index and renders the matching
  TOC row with the accent color.
- Click-to-scroll navigation uses a stable active-reader scrollable ID plus an
  Iced widget operation task to snap to the selected heading's relative block
  anchor.
- `paperview-tui` records rendered line starts for parsed blocks and maps the
  current terminal scroll offset to the latest heading whose block line is at
  or above the viewport.
- The TUI keeps TOC selection local to `ReaderApp`, clamps it after reload, and
  uses the same block-line anchors for active highlighting and jumps.
- `paperview-core::synced_scroll_offset` maps primary scroll offsets to
  secondary scroll offsets by relative progress for split-pane synchronization.

## Open Decisions

- Exact pixel-accurate heading activation is deferred until reader rendering
  exposes actual per-block layout rectangles.
- Independent scroll position persistence is deferred.
- TUI mouse-based TOC navigation is deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI smoke test with a heading-rich document.
