# Inline Span Rendering

## Current Behavior

PaperView preserves common inline Markdown semantics inside heading, paragraph,
list, blockquote, and table-cell content.

- Bold text is represented as strong spans.
- Italic text is represented as emphasis spans.
- Inline code is represented as code spans.
- Links preserve their destination URL.
- Inline images remain visible as Markdown text until image spans are modeled.

The GUI renders heading, paragraph, list, blockquote, and table-cell spans with
Iced rich text. The TUI renders those spans back into Markdown-shaped text.

Document titles, TOC labels, heading slugs, and scroll geometry derive from
plain heading text.

## Implementation Notes

- `paperview-core::parser::InlineSpan` stores text plus strong, emphasis, code,
  and link metadata.
- `paperview-core::parser::elements::inline` owns inline state helpers and
  plain/Markdown text conversion.
- Adjacent spans with identical styling are merged during parsing.
- Headings store `Vec<InlineSpan>` and derive plain labels where navigation or
  window titles need strings.
- Paragraph and blockquote blocks now store `Vec<InlineSpan>`.
- List items now store `Vec<InlineSpan>` per item.
- Table cells now store `Vec<InlineSpan>` per cell.

## Open Decisions

- Decide when GUI links should become clickable commands.
- Decide how nested inline styles should be exposed for export.

## Verification

- Parser tests cover heading, paragraph, list, blockquote, and table-cell span
  preservation.
- TUI tests cover Markdown-shaped inline rendering.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
