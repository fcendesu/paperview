# Inline Span Rendering

## Current Behavior

PaperView preserves common inline Markdown semantics inside paragraph blocks.

- Bold text is represented as strong spans.
- Italic text is represented as emphasis spans.
- Inline code is represented as code spans.
- Links preserve their destination URL.
- Inline images remain visible as Markdown text until image spans are modeled.

The GUI renders paragraph spans with Iced rich text. The TUI renders paragraph
spans back into Markdown-shaped text.

This is a paragraph-only foundation. Headings, list items, blockquotes, and
table cells still use the older string model.

## Implementation Notes

- `paperview-core::parser::InlineSpan` stores text plus strong, emphasis, code,
  and link metadata.
- `paperview-core::parser::elements::inline` owns inline state helpers and
  plain/Markdown text conversion.
- Adjacent spans with identical styling are merged during parsing.
- Paragraph blocks now store `Vec<InlineSpan>`.

## Open Decisions

- Extend inline spans to list items, blockquotes, table cells, and headings.
- Decide when GUI links should become clickable commands.
- Decide how nested inline styles should be exposed for export.

## Verification

- Parser tests cover paragraph span preservation.
- TUI tests cover Markdown-shaped inline rendering.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
