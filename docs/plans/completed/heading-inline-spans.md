# Heading Inline Spans Plan

## Goal

Extend inline span support into Markdown headings while preserving plain-text
document titles, TOC labels, slugs, and scroll geometry.

## Scope

- Store heading content as `InlineSpan` values.
- Derive document titles, TOC labels, slugs, and layout estimates from heading
  plain text.
- Render GUI headings with rich inline text.
- Render TUI headings as Markdown-shaped text.
- Update inline-span feature documentation and progress tracking.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed inline span support for Markdown headings. Heading blocks now store
`InlineSpan` values, the GUI renders styled heading content with rich text, and
the TUI renders Markdown-shaped inline heading content. Document titles, TOC
labels, slugs, and scroll geometry continue to derive from plain heading text.
