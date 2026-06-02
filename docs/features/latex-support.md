# LaTeX Support

## Current Behavior

PaperView preserves LaTeX math source emitted by `pulldown-cmark`.

- Inline math such as `$x + y$` remains visible inside text blocks and is
  represented as structured inline math span metadata.
- Display math such as `$$ E = mc^2 $$` is represented as a dedicated parsed
  document block.
- The GUI renders display math as a source-preserving math panel with a readable
  Unicode-ish preview for common symbols, fractions, roots, Greek letters,
  arrows, set/logic operators, sums/integrals, and compact numeric scripts.
- The TUI renders display math with `$$` delimiters and shows the same
  Unicode-ish readable preview when it improves the source.

This is still a foundation slice. PaperView does not yet fully typeset formulas
or validate LaTeX syntax.

Full `.tex` documents are also not supported yet. PaperView currently accepts
Markdown and plain text files; an Overleaf-style source with `\documentclass`,
packages, custom commands, tables, and layout directives will not render as a
compiled document.

## Implementation Notes

- `paperview-core::parser::Block::Math` stores display math source.
- `paperview-core::parser::InlineSpan` marks inline math spans with `math: true`
  while preserving the visible `$...$` delimiters.
- `paperview-core::parser::elements::math` owns math-source normalization and
  the lightweight readable-preview transform, including common symbol
  replacement, braced fraction/root/vector cleanup, LaTeX spacing command
  cleanup, and single or braced compact script conversion where Unicode has
  clear characters.
- `pulldown-cmark` math events are enabled through the existing parser options.
- Empty paragraph wrappers around standalone display math are discarded during
  parsing.
- `paperview-gui::reader` shows the readable preview above the original display
  math source when the helper can improve the source text.
- `paperview-gui::reader` gives inline math spans a subtle monospace treatment.
- `paperview-tui::render` shows the readable preview above the original display
  math source when the helper can improve the source text and keeps inline math
  Markdown-shaped.
- HTML export emits inline math spans as `code.math.inline` and shows readable
  display-math previews above preserved display math source.
- PDF export includes readable display-math preview text before preserved
  display math source, subject to the current text-first PDF writer's ASCII font
  limitations.

## Open Decisions

- Choose the full native formula rendering path for Markdown math.
- Decide how inline math should participate in future native formula rendering.
- Full `.tex` support will use Tectonic as the integration target. Tectonic is
  a self-contained TeX/LaTeX engine with Rust library and CLI surfaces, making
  it a good fit for rendering existing Overleaf-compatible `.tex` projects into
  PDF/pages without requiring users to install a full TeX Live distribution.
- Non-plan alternatives and why they are not the chosen direction:
  - Shelling out to `latexmk`, `pdflatex`, `xelatex`, or `lualatex` depends on
    an external TeX installation and platform setup.
  - Typst is a different language and would not directly render existing `.tex`
    resumes without conversion.
  - A custom LaTeX parser/renderer is too broad for PaperView's viewer-first
    scope because full documents rely on macros, packages, and layout engines.

## Verification

- Parser tests cover inline math span metadata and display math blocks.
- Core math tests cover readable preview generation for common symbols,
  fractions, roots, Greek letters, arrows, set/logic operators, sums/integrals,
  spacing cleanup, vectors, and compact scripts.
- TUI tests cover display math output.
- Export tests cover readable display-math previews in HTML and PDF output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
