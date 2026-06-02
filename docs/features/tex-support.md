# Tectonic `.tex` Support

## Product Behavior

PaperView does not currently open full `.tex` documents. `.tex` files are now
recognized as a distinct supported file type, but they are not routed through
the Markdown reader. Full `.tex` support will target Overleaf-compatible LaTeX
sources through Tectonic-backed compilation.

The current first implementation slice:

- Accepts `.tex` source files as a distinct file type.
- Keeps `.tex` files out of `Document::open` so they are not parsed as
  Markdown.
- Adds core compile input/artifact/error types.
- Plans the default PDF artifact path as `source.tex` -> `source.pdf`.
- Returns an explicit "Tectonic adapter not implemented yet" compile error.

The next implementation slice should:

- Compile a single entry `.tex` file with Tectonic into a PDF artifact.
- Report compile success, output path, and compiler diagnostics without
  launching a frontend.
- Preserve PaperView's existing Markdown-first reader behavior.

Later slices can expose compiled `.tex` output in the GUI and TUI:

- GUI: open or preview the compiled PDF artifact, then move toward rendered page
  previews if a durable PDF/page renderer is selected.
- TUI: show compile status, diagnostics, and generated artifact paths rather
  than attempting terminal page rendering.

## Implementation Notes

- Tectonic is the chosen full `.tex` engine because it is self-contained,
  supports existing LaTeX/Overleaf-style sources, and has Rust-friendly library
  and CLI integration paths.
- `.tex` support should not route through the Markdown parser. It needs a
  separate core model or artifact path so Markdown assumptions do not leak into
  compiled LaTeX workflows.
- The first core API uses an explicit compile function instead of silently
  compiling during `Document::open`.
- Generated PDFs should be treated as artifacts, similar to export output, so
  users can inspect or open them outside PaperView.
- Diagnostics should remain user-facing and testable: missing packages, syntax
  errors, and Tectonic setup/network failures should produce clear errors.

## Decisions And Gaps

- Decide whether the first implementation uses Tectonic's Rust APIs, a bundled
  CLI invocation, or both behind a small adapter.
- Decide where generated PDFs should live: beside the source file, in a
  `.paperview/` cache directory, or in a temporary directory for check-only
  runs.
- Decide how to handle multi-file LaTeX projects that use `\input`,
  `\include`, images, bibliographies, or custom style files.
- Decide whether GUI preview should open the generated PDF with the platform
  opener first, or wait for an embedded PDF/page preview.
- Full Markdown math formula rendering remains separate from full `.tex`
  document compilation.

## Verification Expectations

Initial `.tex` support should include:

```sh
cargo fmt --all
cargo test -p paperview-core tex
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

When a smoke-test `.tex` fixture exists, verification should also compile it
through the new PaperView entrypoint and remove generated artifacts unless they
are intentionally committed.
