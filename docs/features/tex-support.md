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
- Plans the default PDF artifact path as `source.tex` ->
  `.paperview/tex/source.pdf` under the source file's directory.
- Runs a configurable Tectonic command-line adapter. The default compiler name
  is `tectonic`; callers may provide a custom compiler path for tests or
  bundled runtimes.
- Adds a headless TUI command:
  `cargo run -p paperview-tui -- tex compile <file.tex>`.
- Supports `--open` on that command to open the generated PDF with the platform
  opener after a successful compile.
- Adds a headless `tex clean <file.tex|dir>` command for removing generated
  `.paperview/tex/` artifacts.
- GUI launch, drag-and-drop, and local-link open flows compile `.tex` files and
  open the generated PDF externally as the first desktop preview behavior.
  GUI compilation runs through Iced tasks so the app can show `Compiling ...`
  instead of blocking on Tectonic.
- Honors the optional `tex_compiler_path` config setting when the Tectonic
  executable lives outside `PATH`.
- Includes `docs/fixtures/minimal.tex` as a local smoke fixture for manual
  end-to-end compile checks.
- Reports missing compiler, compiler failure, missing output PDF, and output
  write errors as explicit compile errors.

The next implementation slice should:

- Decide whether PaperView should bundle the Tectonic binary or continue with
  configured/PATH discovery.
- Preserve PaperView's existing Markdown-first reader behavior.

Later slices can expose compiled `.tex` output in the GUI and TUI:

- GUI: preview the compiled PDF artifact once a durable PDF/page renderer is
  selected.
- TUI: show compile status, diagnostics, and generated artifact paths rather
  than attempting terminal page rendering.

## Implementation Notes

- Tectonic is the chosen full `.tex` engine because it is self-contained and
  supports existing LaTeX/Overleaf-style sources.
- The current adapter uses the Tectonic CLI shape: `tectonic --outdir <dir>
  <entry.tex>`. This avoids linking the Rust crate into PaperView for now.
- `.tex` support should not route through the Markdown parser. It needs a
  separate core model or artifact path so Markdown assumptions do not leak into
  compiled LaTeX workflows.
- The first core API uses an explicit compile function instead of silently
  compiling during `Document::open`.
- The headless command prints the generated PDF path and includes Tectonic
  diagnostics when the compiler reports them.
- `tex clean <file.tex>` removes that file's managed PDF artifact. `tex clean
  <dir>` removes `<dir>/.paperview/tex/`.
- The `--open` option delegates the generated PDF artifact to the existing
  platform opener helper. It does not embed PDF rendering in PaperView.
- GUI `.tex` opening uses the same external-open behavior for now and reports
  `Compiling ...` followed by the source/output artifact path in shell status
  text. Launch-time `.tex` startup returns an initial compile task through the
  Iced boot function.
- Generated PDFs should be treated as artifacts, similar to export output, so
  users can inspect or open them outside PaperView.
- Generated PDFs and Tectonic byproducts under `docs/fixtures/.paperview/` are
  ignored so smoke checks do not dirty the repository.
- Diagnostics should remain user-facing and testable: missing packages, syntax
  errors, and Tectonic setup/network failures should produce clear errors.

## Decisions And Gaps

- The Rust crate integration was evaluated first, but `tectonic 0.16.9` pulled
  in native bridge crates that required system `graphite2` discovery through
  `pkg-config` on macOS. The CLI adapter is the selected first implementation
  path until bundling/runtime policy is settled.
- The current default generated PDF path is under `.paperview/tex/` beside the
  source file's directory. Explicit output paths are still available through the
  core API for tests or future advanced commands.
- The headless command uses `tex_compiler_path` from config when present, and
  otherwise falls back to the default `tectonic` executable name.
- Decide how to handle multi-file LaTeX projects that use `\input`,
  `\include`, images, bibliographies, or custom style files.
- Decide how embedded GUI PDF/page preview should work after the external-open
  bridge.
- Full Markdown math formula rendering remains separate from full `.tex`
  document compilation.

## Verification Expectations

Initial `.tex` support should include:

```sh
cargo fmt --all
cargo test -p paperview-core tex
cargo test -p paperview-tui tex
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

When Tectonic is installed, run the optional end-to-end smoke check:

```sh
cargo run -p paperview-tui -- tex compile docs/fixtures/minimal.tex
```

The generated `docs/fixtures/.paperview/tex/minimal.pdf` and Tectonic byproducts
are ignored and should remain uncommitted.

The latest local smoke result:

- Date: 2026-06-02.
- Tectonic: `Tectonic 0.16.9` at `/opt/homebrew/bin/tectonic`.
- Command:
  `cargo run -p paperview-tui -- tex compile docs/fixtures/minimal.tex`.
- Result: generated a valid `docs/fixtures/minimal.pdf`, then removed it. This
  result predates the managed `.paperview/tex/` artifact path.
