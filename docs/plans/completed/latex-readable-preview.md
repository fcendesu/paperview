# LaTeX Readable Preview Plan

## Goal

Improve GUI display math readability with a lightweight native preview while
preserving LaTeX source exactly enough for technical review.

## Scope

- Add a core helper that converts common LaTeX display-math tokens to readable
  Unicode-ish text.
- Keep `Block::Math` as source-preserving data.
- Render the preview in the GUI math panel above the original source.
- Keep TUI display math source output unchanged.
- Leave full formula typesetting, validation, and structured inline math spans
  deferred.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI launch smoke check with display math

## Outcome

Completed a lightweight GUI readable-preview layer for display math. Core now
translates common LaTeX symbols, fractions, square roots, and numeric scripts
into Unicode-ish preview text. The GUI shows that preview above the preserved
source when it improves readability. TUI output remains source-preserving, and
full formula typesetting remains deferred.
