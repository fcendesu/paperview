# Table Rendering

## Current Behavior

PaperView parses GitHub-style Markdown tables into structured document blocks.

- Column alignments are preserved from the Markdown delimiter row.
- Header cells and body rows are stored separately in `paperview-core`.
- Inline formatting inside table cells is preserved.
- The GUI renders tables as bordered reader panels with shaded headers and
  shared responsive column proportions across each table.
- The TUI renders aligned Markdown-style table text and wraps very long cell
  content across continuation rows instead of allowing one cell to stretch the
  whole table indefinitely.

This is a readable first pass. It does not yet support horizontal scrolling,
row selection, copy interactions, or user-controlled column resizing.

## Implementation Notes

- `paperview-core::parser::Block::Table` stores alignments, header cells, and
  body rows.
- `paperview-core::parser::elements::table` owns table alignment conversion and
  table-specific helpers.
- The parser consumes `pulldown-cmark` table events directly.
- Table cells use `InlineSpan`, and renderers measure columns from rendered
  cell text.
- GUI table columns use shared fill portions derived from the widest cell in
  each column, with readable minimums and maximums so long cells wrap inside the
  reader width instead of forcing a wider table.
- TUI table columns are capped at a readable width, with long words split and
  continuation lines padded/aligned within the same table shape.
- GUI table heights are estimated for TOC scroll mapping.

## Open Decisions

- Decide whether wide GUI tables should eventually support explicit horizontal
  scrolling in addition to compact responsive columns.
- Decide whether the TUI should expose a user-configurable table width cap.

## Verification

- Parser tests cover table structure, alignment preservation, and inline cell
  spans.
- GUI tests cover shared bounded responsive column portions.
- TUI tests cover aligned table output, Markdown-shaped inline cell output, and
  wrapped long-cell output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
