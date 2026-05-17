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

## Implementation Notes

- `paperview-gui` stores the active TOC item as a heading block index.
- The current mapping uses scroll progress against parsed document block
  positions. This is deterministic and cheap, but it is not pixel-perfect
  heading geometry yet.
- `paperview-core` already exposes heading block indices through `TocItem`, so
  no new core data model was needed for the first GUI slice.
- `reader::view_with_scroll` adapts Iced scrollable viewport changes into a
  normalized scroll-progress message for the app state.
- `navigation::view` receives the active block index and renders the matching
  TOC row with the accent color.
- Click-to-scroll navigation uses a stable active-reader scrollable ID plus an
  Iced widget operation task to snap to the selected heading's relative block
  position.

## Open Decisions

- Pixel-accurate heading activation is deferred until reader rendering exposes
  per-block layout metadata.
- Split-pane scroll synchronization is deferred.
- Independent scroll position persistence is deferred.
- TUI scroll synchronization is deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI smoke test with a heading-rich document.
