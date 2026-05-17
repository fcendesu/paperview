# LaTeX Support

## Current Behavior

PaperView preserves LaTeX math source emitted by `pulldown-cmark`.

- Inline math such as `$x + y$` remains visible inside text blocks.
- Display math such as `$$ E = mc^2 $$` is represented as a dedicated parsed
  document block.
- The GUI renders display math as a source-preserving math panel.
- The TUI renders display math with `$$` delimiters.

This is a foundation slice. PaperView does not yet typeset formulas, validate
LaTeX syntax, or provide a dedicated inline math span model for rich frontend
styling.

## Implementation Notes

- `paperview-core::parser::Block::Math` stores display math source.
- `paperview-core::parser::elements::math` owns math-source normalization.
- `pulldown-cmark` math events are enabled through the existing parser options.
- Empty paragraph wrappers around standalone display math are discarded during
  parsing.

## Open Decisions

- Choose the native formula rendering path for the GUI.
- Decide whether inline math should become a structured inline span model once
  richer text rendering exists.
- Decide whether the TUI should keep source rendering only or add Unicode-ish
  preview affordances for simple formulas.

## Verification

- Parser tests cover inline-source preservation and display math blocks.
- TUI tests cover display math output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
