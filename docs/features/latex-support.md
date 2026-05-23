# LaTeX Support

## Current Behavior

PaperView preserves LaTeX math source emitted by `pulldown-cmark`.

- Inline math such as `$x + y$` remains visible inside text blocks.
- Display math such as `$$ E = mc^2 $$` is represented as a dedicated parsed
  document block.
- The GUI renders display math as a source-preserving math panel with a readable
  Unicode-ish preview for common symbols, fractions, roots, and numeric scripts.
- The TUI renders display math with `$$` delimiters and shows the same
  Unicode-ish readable preview when it improves the source.

This is still a foundation slice. PaperView does not yet fully typeset formulas,
validate LaTeX syntax, or provide a dedicated inline math span model for rich
frontend styling.

## Implementation Notes

- `paperview-core::parser::Block::Math` stores display math source.
- `paperview-core::parser::elements::math` owns math-source normalization and
  the lightweight readable-preview transform.
- `pulldown-cmark` math events are enabled through the existing parser options.
- Empty paragraph wrappers around standalone display math are discarded during
  parsing.
- `paperview-gui::reader` shows the readable preview above the original display
  math source when the helper can improve the source text.
- `paperview-tui::render` shows the readable preview above the original display
  math source when the helper can improve the source text.

## Open Decisions

- Choose the full native formula rendering path for the GUI.
- Decide whether inline math should become a structured inline span model once
  richer text rendering exists.

## Verification

- Parser tests cover inline-source preservation and display math blocks.
- Core math tests cover readable preview generation.
- TUI tests cover display math output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
