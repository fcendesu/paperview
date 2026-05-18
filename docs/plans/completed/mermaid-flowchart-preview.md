# Mermaid Flowchart Preview Plan

## Goal

Add a native preview for simple Mermaid flowchart diagrams without introducing a
browser or JavaScript rendering dependency.

## Scope

- Parse common Mermaid `graph` and `flowchart` edge lines in core.
- Preserve the existing source-preserving `Block::Diagram` model.
- Render parsed flowchart edges as a readable GUI preview.
- Keep GUI source text visible for unsupported syntax and debugging.
- Keep TUI source output unchanged.
- Leave full Mermaid syntax support, layout engines, and export assets deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI launch smoke check with a simple Mermaid flowchart

## Outcome

Completed a native preview foundation for simple Mermaid flowcharts. Core now
parses common `graph` and `flowchart` edge lines into preview edges, and the GUI
renders those edges as native node rows while keeping Mermaid source visible.
TUI output remains source-preserving. Full Mermaid syntax, layout, validation,
and export assets remain deferred.
