# Table Cell Inline Spans Plan

## Goal

Extend inline span support into Markdown table cells while preserving the
existing table structure and alignment behavior.

## Scope

- Store table header and body cells as inline spans.
- Preserve bold, italic, inline code, links, inline math text, and inline image
  Markdown text in table cells.
- Keep table width calculations based on plain cell text.
- Render GUI table cells with rich text.
- Render TUI table cells as Markdown-shaped text.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed inline span support for Markdown table cells. Header and body cells
now store `InlineSpan` values, GUI tables render rich cell content, and TUI
tables render Markdown-shaped inline cell output while measuring columns from
plain text. Heading inline spans remain deferred.
