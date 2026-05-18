# GUI Image Previews Plan

## Goal

Render standalone local bitmap images in the GUI reader while preserving the
existing metadata fallback for remote, missing, or unresolved images.

## Scope

- Enable Iced image widget support for the GUI crate.
- Resolve relative standalone image URLs against the active document path.
- Render local image files with a constrained preview in the GUI reader.
- Keep alt text, title, and path metadata visible around previews.
- Leave remote fetching, inline image spans, and click-to-zoom deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI launch smoke check with a local image

## Outcome

Completed GUI local bitmap image previews. The GUI crate enables Iced image
widgets, resolves standalone relative image URLs against the active document
path, and renders existing local image files as constrained previews with
metadata. Missing, remote, and unresolved images keep the metadata fallback.
