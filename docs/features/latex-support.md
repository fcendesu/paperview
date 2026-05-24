# LaTeX Support

## Current Behavior

PaperView preserves LaTeX math source emitted by `pulldown-cmark`.

- Inline math such as `$x + y$` remains visible inside text blocks and is
  represented as structured inline math span metadata.
- Display math such as `$$ E = mc^2 $$` is represented as a dedicated parsed
  document block.
- The GUI renders display math as a source-preserving math panel with a readable
  Unicode-ish preview for common symbols, fractions, roots, and numeric scripts.
- The TUI renders display math with `$$` delimiters and shows the same
  Unicode-ish readable preview when it improves the source.

This is still a foundation slice. PaperView does not yet fully typeset formulas
or validate LaTeX syntax.

## Implementation Notes

- `paperview-core::parser::Block::Math` stores display math source.
- `paperview-core::parser::InlineSpan` marks inline math spans with `math: true`
  while preserving the visible `$...$` delimiters.
- `paperview-core::parser::elements::math` owns math-source normalization and
  the lightweight readable-preview transform.
- `pulldown-cmark` math events are enabled through the existing parser options.
- Empty paragraph wrappers around standalone display math are discarded during
  parsing.
- `paperview-gui::reader` shows the readable preview above the original display
  math source when the helper can improve the source text.
- `paperview-gui::reader` gives inline math spans a subtle monospace treatment.
- `paperview-tui::render` shows the readable preview above the original display
  math source when the helper can improve the source text and keeps inline math
  Markdown-shaped.
- HTML export emits inline math spans as `code.math.inline`.

## Open Decisions

- Choose the full native formula rendering path for the GUI.
- Decide how inline math should participate in future native formula rendering.

## Verification

- Parser tests cover inline math span metadata and display math blocks.
- Core math tests cover readable preview generation.
- TUI tests cover display math output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
