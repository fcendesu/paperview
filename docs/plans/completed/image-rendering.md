# Image Rendering Plan

## Goal

Add first-pass Markdown image support across the shared parser, GUI reader, and
TUI renderer.

## Scope

- Parse standalone Markdown images into structured image blocks.
- Preserve inline images as readable Markdown text inside surrounding content.
- Render GUI image metadata panels.
- Render TUI image Markdown text.
- Document current behavior and deferred bitmap preview support.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed first-pass Markdown image rendering. Standalone images are parsed into
`Block::Image`, inline images remain visible as Markdown text, the GUI renders
image metadata panels, and the TUI renders Markdown image syntax. Bitmap preview,
relative path resolution, remote fetching, and click-to-zoom remain deferred.
