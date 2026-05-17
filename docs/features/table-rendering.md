# Table Rendering

## Current Behavior

PaperView parses GitHub-style Markdown tables into structured document blocks.

- Column alignments are preserved from the Markdown delimiter row.
- Header cells and body rows are stored separately in `paperview-core`.
- The GUI renders tables as bordered reader panels with shaded headers.
- The TUI renders aligned Markdown-style table text.

This is a readable first pass. It does not yet support horizontal scrolling,
rich inline spans inside cells, row selection, copy interactions, or responsive
column resizing.

## Implementation Notes

- `paperview-core::parser::Block::Table` stores alignments, header cells, and
  body rows.
- `paperview-core::parser::elements::table` owns table alignment conversion and
  cell text normalization.
- The parser consumes `pulldown-cmark` table events directly.
- GUI table heights are estimated for TOC scroll mapping.

## Open Decisions

- Decide whether wide GUI tables should scroll horizontally or compact columns.
- Decide how rich inline formatting inside table cells should be represented
  once inline spans exist.
- Decide whether TUI table rendering should wrap long cells or keep one row per
  Markdown row.

## Verification

- Parser tests cover table structure and alignment preservation.
- TUI tests cover aligned table output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
