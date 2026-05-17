# List And Blockquote Inline Spans Plan

## Goal

Extend the inline span foundation from paragraphs into list items and
blockquotes.

## Scope

- Store list item content as inline spans.
- Store blockquote content as inline spans.
- Preserve bold, italic, inline code, links, inline math text, and inline image
  Markdown text in those blocks.
- Render list and blockquote spans in GUI rich text.
- Render list and blockquote spans in TUI Markdown-shaped text.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed inline span support for list items and blockquotes. The shared parser
now stores list item and blockquote content as `InlineSpan` values, the GUI
renders them with rich text, and the TUI renders Markdown-shaped inline output.
Table cells and headings remain deferred.
