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
Iced rich text. Link spans are clickable in the GUI and open through the
platform default opener. Relative GUI links resolve from the active document's
parent directory when the document has a path. The TUI renders spans back into
Markdown-shaped text and keeps links display-only.

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
- `paperview-gui::reader` attaches Iced rich-text link metadata to link spans
  and emits a GUI message when a link is clicked.
- `paperview-gui::app` resolves relative link targets against the active
  document path and delegates opening to the platform default opener.

## Open Decisions

- Decide how in-document anchor links should navigate to headings.
- Decide how nested inline styles should be exposed for export.

## Verification

- Parser tests cover heading, paragraph, list, blockquote, and table-cell span
  preservation.
- GUI tests cover relative link target resolution and empty-link rejection.
- TUI tests cover Markdown-shaped inline rendering.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
