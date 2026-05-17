# Mermaid Foundation Plan

## Goal

Add the first PaperView Mermaid support by recognizing Mermaid fenced code blocks
as semantic diagram blocks and rendering their source visibly in both frontends.

## Scope

- Detect fenced code blocks whose language is `mermaid`.
- Represent Mermaid content as a dedicated parsed document block.
- Render Mermaid source-preserving panels in GUI and TUI.
- Document current behavior and deferred native diagram rendering.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed the source-preserving Mermaid foundation. Mermaid fenced code blocks
are parsed as `Block::Diagram`, the GUI renders them in diagram panels, and the
TUI preserves the fenced source shape. Native diagram rendering remains
deferred.
