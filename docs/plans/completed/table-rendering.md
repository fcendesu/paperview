# Table Rendering Plan

## Goal

Add first-class Markdown table support across the shared parser, GUI reader, and
TUI renderer.

## Scope

- Parse `pulldown-cmark` table events into a structured document block.
- Preserve column alignments, headers, and body rows in `paperview-core`.
- Render readable table panels in the GUI.
- Render aligned plain-text tables in the TUI.
- Document current behavior and follow-up gaps.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed first-pass Markdown table rendering. Tables are parsed into
`Block::Table` with alignments, headers, and body rows. The GUI renders bordered
table panels with shaded headers, and the TUI renders aligned Markdown-style
tables. Wide-table scrolling, responsive sizing, and rich inline cell spans
remain deferred.
