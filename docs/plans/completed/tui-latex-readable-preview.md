# TUI LaTeX Readable Preview

## Goal

Give terminal readers the same lightweight display-math readability affordance
already available in the GUI.

## Completed

- Reused `paperview-core::parser::elements::math::readable_preview` in the TUI
  renderer.
- Rendered the readable preview above the original display math source when the
  helper can improve the source text.
- Preserved the original `$$` source block for copyability and fidelity.
- Added focused TUI render coverage for previewed and source-only math output.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core math`
- `cargo test -p paperview-tui latex`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
