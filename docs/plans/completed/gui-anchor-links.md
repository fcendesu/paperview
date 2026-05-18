# GUI Anchor Links Plan

## Goal

Make clicked GUI in-document links such as `[Usage](#usage)` jump to matching
Markdown headings in the active document.

## Scope

- Detect clicked links whose target starts with `#`.
- Match anchor targets against `ParsedDocument::toc()` slugs.
- Reuse the existing GUI reader scroll task used by TOC clicks.
- Update the active TOC item when an anchor jump succeeds.
- Show a status error for missing anchors.
- Document the behavior and remaining link limitations.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI launch smoke check with anchor links

## Outcome

Completed GUI in-document anchor links. Clicked `#slug` links now match active
document TOC slugs, update the active TOC item, and reuse the GUI reader scroll
task used by TOC navigation. Missing anchors report a status error.
