# Table Rendering

## Current Behavior

PaperView parses GitHub-style Markdown tables into structured document blocks.

- Column alignments are preserved from the Markdown delimiter row.
- Header cells and body rows are stored separately in `paperview-core`.
- Inline formatting inside table cells is preserved.
- The GUI renders tables as bordered reader panels with shaded headers.
- The TUI renders aligned Markdown-style table text and wraps very long cell
  content across continuation rows instead of allowing one cell to stretch the
  whole table indefinitely.

This is a readable first pass. It does not yet support horizontal scrolling,
row selection, copy interactions, or responsive column resizing.

## Implementation Notes

- `paperview-core::parser::Block::Table` stores alignments, header cells, and
  body rows.
- `paperview-core::parser::elements::table` owns table alignment conversion and
  table-specific helpers.
- The parser consumes `pulldown-cmark` table events directly.
- Table cells use `InlineSpan`, and renderers measure columns from rendered
  cell text.
- TUI table columns are capped at a readable width, with long words split and
  continuation lines padded/aligned within the same table shape.
- GUI table heights are estimated for TOC scroll mapping.

## Open Decisions

- Decide whether wide GUI tables should scroll horizontally or compact columns.
- Decide whether the TUI should expose a user-configurable table width cap.

## Verification

- Parser tests cover table structure, alignment preservation, and inline cell
  spans.
- TUI tests cover aligned table output, Markdown-shaped inline cell output, and
  wrapped long-cell output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
