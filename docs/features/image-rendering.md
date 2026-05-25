# Image Rendering

## Current Behavior

PaperView preserves Markdown image metadata.

- Standalone images are parsed into structured image blocks with alt text, URL,
  and title.
- Inline images remain visible as Markdown text inside surrounding paragraphs,
  list items, or table cells.
- The GUI renders standalone local bitmap images as constrained previews with
  metadata, including decoded dimensions for common image headers.
- The GUI fetches `http://` and `https://` standalone images for constrained
  remote previews. Loaded remote previews include decoded dimensions for common
  image headers. Loading and failed remote previews keep the metadata panel
  visible with status text.
- Missing and unresolved non-HTTP images fall back to metadata panels.
- The TUI renders standalone images in Markdown image syntax plus a metadata
  line that identifies remote images, unresolved paths, missing local files, or
  existing local files with byte size and decoded PNG/JPEG/GIF/WebP dimensions
  when available.
- PDF export renders standalone images as text placeholders with the same
  local, missing, unresolved, and remote status categories, including local file
  size and decoded PNG/JPEG/GIF/WebP dimensions when available.

This is still a first pass. PaperView does not yet drive preview layout from
decoded dimensions or support click-to-zoom.

## Implementation Notes

- `paperview-core::parser::Block::Image` stores `alt`, `url`, and `title`.
- `paperview-core::parser::elements::image` owns image alt normalization and
  Markdown text reconstruction.
- Standalone images are promoted from image-only paragraphs into image blocks.
- Inline images stay textual until PaperView has image-specific inline spans.
- The GUI uses Iced image widgets for local image files.
- `paperview-core::parser::elements::image` owns dependency-light PNG, JPEG,
  GIF, and WebP header dimension parsing.
- The GUI uses shared core image dimension helpers for local files and loaded
  remote bytes.
- Relative standalone image URLs resolve against the active document path in GUI
  previews, TUI local metadata lines, and PDF image placeholders.
- Remote image fetching stays in `paperview-gui`; core remains responsible for
  image metadata parsing only.
- Remote image bytes are cached in GUI state for the current session and are
  capped at 10 MB per image.

## Open Decisions

- Decide whether image previews should use decoded dimensions for more precise
  layout estimates instead of metadata-only display.
- Decide whether remote dimensions should be recorded outside GUI session state
  for future export workflows.

## Verification

- Parser tests cover standalone image blocks and inline image preservation.
- Core tests cover PNG/JPEG/GIF/WebP dimension parsing.
- GUI tests cover relative image path resolution, remote URL classification,
  and remote image loading placeholders.
- TUI tests cover Markdown image output plus local, missing, unresolved, and
  remote metadata lines, including local dimensions.
- PDF export tests cover local, missing, and remote image placeholder metadata,
  including local dimensions.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
