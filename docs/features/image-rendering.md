# Image Rendering

## Current Behavior

PaperView preserves Markdown image metadata.

- Standalone images are parsed into structured image blocks with alt text, URL,
  and title.
- Inline images remain visible as Markdown text inside surrounding paragraphs,
  list items, or table cells.
- The GUI renders standalone local bitmap images as constrained previews with
  metadata. Missing, remote, and unresolved images fall back to metadata panels.
- The TUI renders standalone images in Markdown image syntax.

This is still a first pass. PaperView does not yet fetch remote images, resize
images from actual decoded dimensions, or support click-to-zoom.

## Implementation Notes

- `paperview-core::parser::Block::Image` stores `alt`, `url`, and `title`.
- `paperview-core::parser::elements::image` owns image alt normalization and
  Markdown text reconstruction.
- Standalone images are promoted from image-only paragraphs into image blocks.
- Inline images stay textual until PaperView has image-specific inline spans.
- The GUI uses Iced image widgets for local image files.
- Relative standalone image URLs resolve against the active document path.

## Open Decisions

- Decide whether remote image fetching belongs in core or a GUI-specific
  loading layer.
- Decide whether image previews should use decoded dimensions for more precise
  layout estimates.
- Decide whether TUI should expose file metadata for local images.

## Verification

- Parser tests cover standalone image blocks and inline image preservation.
- GUI tests cover relative image path resolution and remote image fallback.
- TUI tests cover Markdown image output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
