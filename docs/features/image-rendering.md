# Image Rendering

## Current Behavior

PaperView preserves Markdown image metadata.

- Standalone images are parsed into structured image blocks with alt text, URL,
  and title.
- Inline images remain visible as Markdown text inside surrounding paragraphs,
  list items, or table cells.
- The GUI renders standalone images as metadata panels.
- The TUI renders standalone images in Markdown image syntax.

This is a first pass. PaperView does not yet decode bitmap files, fetch remote
images, resize images responsively, or support click-to-zoom.

## Implementation Notes

- `paperview-core::parser::Block::Image` stores `alt`, `url`, and `title`.
- `paperview-core::parser::elements::image` owns image alt normalization and
  Markdown text reconstruction.
- Standalone images are promoted from image-only paragraphs into image blocks.
- Inline images stay textual until PaperView has a richer inline-span model.

## Open Decisions

- Decide whether GUI image preview should use Iced image widgets directly or a
  small rendering abstraction that can handle local files and remote URLs.
- Decide how relative image URLs should resolve against the active document
  path.
- Decide whether TUI should expose file metadata for local images.

## Verification

- Parser tests cover standalone image blocks and inline image preservation.
- TUI tests cover Markdown image output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
