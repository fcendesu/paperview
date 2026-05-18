# Basic Markdown Rendering

## Product Behavior

PaperView now has a shared core Markdown parse model that frontends can render without depending directly on `pulldown-cmark` events.

The initial block model supports:

- Headings
- Paragraphs
- Blockquotes
- Fenced code blocks with optional language labels
- Ordered and unordered lists
- Horizontal rules
- Heading, paragraph, list, blockquote, and table-cell inline spans for bold,
  italic, inline code, and links

This remains an early rendering foundation, not a polished final UI. TUI rendering uses a first-pass Ratatui shell with a scrollable reader. GUI rendering uses native Iced widgets in a first-pass PaperView reader shell.

## Implementation Notes

- `paperview-core::parser::parse_markdown` uses `pulldown-cmark`.
- `ParsedDocument::title` returns the first level-one heading.
- Heading, paragraph, list, blockquote, and table-cell inline styling is preserved with `InlineSpan`.
- Frontends use PaperView's own `HeadingLevel` type rather than depending on `pulldown-cmark` event types.
- Markdown element modules remain under `paperview-core/src/parser/elements/`; focused per-element implementations should move there as rendering requirements deepen.

## Open Decisions

- Add tables and task lists as dedicated element modules rather than expanding the parser orchestrator indefinitely.
- Improve exact scroll restoration and richer inline span interactions as the reader matures.

## Verification Expectations

Run parser-focused tests with:

```sh
cargo test -p paperview-core parser
```

Run the workspace checks before finishing parser changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
