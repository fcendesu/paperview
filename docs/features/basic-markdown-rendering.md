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

This remains an early rendering foundation, not a polished final UI. TUI rendering outputs simple terminal Markdown-like text. GUI rendering uses native Iced widgets in a first-pass PaperView reader shell.

## Implementation Notes

- `paperview-core::parser::parse_markdown` uses `pulldown-cmark`.
- `ParsedDocument::title` returns the first level-one heading.
- Inline styling is flattened into block text for now so the first AST stays small.
- Frontends use PaperView's own `HeadingLevel` type rather than depending on `pulldown-cmark` event types.
- Markdown element modules remain under `paperview-core/src/parser/elements/`; focused per-element implementations should move there as rendering requirements deepen.

## Open Decisions

- Preserve structured inline spans before implementing rich GUI/TUI text styling.
- Add tables and task lists as dedicated element modules rather than expanding the parser orchestrator indefinitely.
- Preserve scroll position and richer inline spans before adding live reload and TOC synchronization.

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
