# LaTeX Math Foundation Plan

## Goal

Add the first native PaperView support for LaTeX math by preserving math source in
the parsed document model and rendering it visibly in both frontends.

## Scope

- Parse inline and display math emitted by `pulldown-cmark`.
- Preserve inline math inside text blocks with dollar delimiters.
- Represent display math as a dedicated block in `paperview-core`.
- Render display math source in GUI and TUI.
- Document current behavior and deferred full typesetting.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI/TUI launch smoke checks

## Outcome

Completed the source-preserving LaTeX foundation. Inline math remains visible in
text, display math is modeled as `Block::Math`, and both frontends render display
math source. Full formula typesetting remains deferred.
