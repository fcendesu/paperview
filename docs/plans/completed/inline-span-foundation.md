# Inline Span Foundation Plan

## Goal

Add a first structured inline span model for paragraph content so PaperView can
preserve and render common inline Markdown semantics.

## Scope

- Represent paragraph text as inline spans in `paperview-core`.
- Preserve bold, italic, inline code, and links for paragraphs.
- Keep headings, lists, blockquotes, and tables on the existing string model for
  this slice.
- Render paragraph spans in the GUI with basic rich text styling.
- Render paragraph spans in the TUI as Markdown-shaped text.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed the paragraph inline span foundation. Paragraph blocks now store
`InlineSpan` values for bold, italic, inline code, and links. The GUI renders
paragraphs with Iced rich text, and the TUI renders paragraph spans back into
Markdown-shaped text. Inline spans for lists, tables, blockquotes, and headings
remain deferred.
