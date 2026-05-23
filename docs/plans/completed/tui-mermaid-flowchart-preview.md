# TUI Mermaid Flowchart Preview

## Goal

Give terminal readers a quick structural preview for simple Mermaid flowcharts
while preserving the original source.

## Completed

- Reused `paperview-core::parser::elements::diagram::flowchart_preview` in the
  TUI renderer.
- Rendered preview direction and edge rows above supported Mermaid source
  blocks.
- Kept unsupported Mermaid diagrams source-only.
- Added focused TUI render tests for previewed and unsupported diagrams.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core diagram`
- `cargo test -p paperview-tui mermaid`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
