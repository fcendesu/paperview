# Image Rendering

## Current Behavior

PaperView preserves Markdown image metadata.

- Standalone images are parsed into structured image blocks with alt text, URL,
  and title.
- Inline images remain visible as Markdown text inside surrounding paragraphs,
  list items, or table cells.
- The GUI renders standalone local bitmap images as constrained previews with
  metadata.
- The GUI fetches `http://` and `https://` standalone images for constrained
  remote previews. Loading and failed remote previews keep the metadata panel
  visible with status text.
- Missing and unresolved non-HTTP images fall back to metadata panels.
- The TUI renders standalone images in Markdown image syntax plus a metadata
  line that identifies remote images, unresolved paths, missing local files, or
  existing local files with byte size.

This is still a first pass. PaperView does not yet resize images from actual
decoded dimensions or support click-to-zoom.

## Implementation Notes

- `paperview-core::parser::Block::Image` stores `alt`, `url`, and `title`.
- `paperview-core::parser::elements::image` owns image alt normalization and
  Markdown text reconstruction.
- Standalone images are promoted from image-only paragraphs into image blocks.
- Inline images stay textual until PaperView has image-specific inline spans.
- The GUI uses Iced image widgets for local image files.
- Relative standalone image URLs resolve against the active document path in GUI
  previews and TUI local metadata lines.
- Remote image fetching stays in `paperview-gui`; core remains responsible for
  image metadata parsing only.
- Remote image bytes are cached in GUI state for the current session and are
  capped at 10 MB per image.

## Open Decisions

- Decide whether image previews should use decoded dimensions for more precise
  layout estimates.
- Decide whether TUI image metadata should show dimensions if a future
  dependency-light decoding path exists.

## Verification

- Parser tests cover standalone image blocks and inline image preservation.
- GUI tests cover relative image path resolution, remote URL classification, and
  remote image loading placeholders.
- TUI tests cover Markdown image output plus local, missing, unresolved, and
  remote metadata lines.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
