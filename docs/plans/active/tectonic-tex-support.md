# Tectonic `.tex` Support Plan

## Goal

Add a Tectonic-backed path for full `.tex` documents while keeping PaperView's
Markdown reader fast, native, and quiet by default.

The first implementation should prove compilation as a core/headless capability
before GUI or TUI preview polish.

## Scope

- Add a core `.tex` compile/check API backed by Tectonic.
- Add `.tex` file-type recognition without forcing `.tex` through Markdown
  parsing.
- Produce a PDF artifact and user-facing diagnostics for a single entry `.tex`
  file.
- Add a headless command path to compile/check `.tex` files.
- Document the chosen architecture, verification, and remaining preview gaps.

Out of scope for the first slice:

- Embedded GUI PDF/page preview.
- Terminal rendering of compiled PDF pages.
- Multi-file project management beyond what Tectonic can resolve from the entry
  file's working directory.
- Bibliography workflows and custom package management beyond Tectonic's normal
  behavior.
- Full formula typesetting for Markdown math blocks.
- Knowledge Graph behavior.

## Affected Paths

- `crates/paperview-core/src/`
- `crates/paperview-tui/src/main.rs`
- `docs/features/tex-support.md`
- `docs/features/file-opening.md`
- `docs/features/latex-support.md`
- `docs/TASKS.md`
- `docs/PRD.md`
- `docs/arch/INDEX.md`
- `docs/quality/CHECKS.md`

## Implementation Steps

1. Decide the adapter shape for Tectonic integration.
2. Add core types for `.tex` compile input, output artifact, and diagnostics.
3. Add focused core tests around file-type recognition, artifact path planning,
   and diagnostic formatting.
4. Add the first compile/check implementation.
5. Add a headless CLI entrypoint for `.tex` compile/check.
6. Add a small `.tex` fixture or smoke-test path that exercises a minimal
   document.
7. Update docs and tracker state.

## Verification Plan

- `cargo fmt --all`
- `cargo test -p paperview-core tex`
- `cargo test -p paperview-tui tex`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Smoke compile a minimal `.tex` fixture through the new PaperView entrypoint.

## Progress

- 2026-06-02: Plan opened after selecting Tectonic as the full `.tex` support
  direction.
- 2026-06-02: Added `.tex` recognition as a distinct file type, prevented
  `.tex` from being opened as a Markdown `Document`, and introduced core
  compile input/artifact/error types with default PDF artifact path planning.
  The real Tectonic adapter and headless command remain next.
- 2026-06-02: Evaluated `tectonic 0.16.9` as a Rust crate, but the build
  required system `graphite2` discovery through `pkg-config` on macOS. Switched
  the first adapter to a configurable Tectonic CLI invocation behind the same
  `compile_tex` API. Core tests cover command success, compiler failure,
  missing compiler diagnostics, missing output diagnostics, and custom output
  path handling.
- 2026-06-02: Added `paperview-tui tex compile <file.tex>` as the first
  headless entrypoint over the core Tectonic adapter. The command prints the
  generated PDF path and compiler diagnostics when available. Bundling policy,
  GUI/TUI preview behavior, and a real Tectonic smoke fixture remain open.
- 2026-06-02: Added optional `tex_compiler_path` config support so the
  headless command can invoke a custom Tectonic executable. Binary bundling,
  GUI/TUI preview behavior, and a real Tectonic smoke fixture remain open.
- 2026-06-02: Added `docs/fixtures/minimal.tex` plus ignored generated fixture
  artifacts for optional end-to-end smoke checks through
  `paperview-tui tex compile`. Binary bundling and GUI/TUI preview behavior
  remain open.
- 2026-06-02: Ran the real smoke check with `Tectonic 0.16.9` at
  `/opt/homebrew/bin/tectonic`:
  `cargo run -p paperview-tui -- tex compile docs/fixtures/minimal.tex`.
  PaperView generated a valid `docs/fixtures/minimal.pdf`; the generated PDF
  was removed afterward. Binary bundling and GUI/TUI preview behavior remain
  open.
- 2026-06-02: Added `--open` to `paperview-tui tex compile` so a successful
  compile can hand the generated PDF to the platform opener. Embedded GUI/TUI
  PDF preview remains open.
- 2026-06-02: Added the first GUI `.tex` open behavior for launch,
  drag-and-drop, and local document links: compile through Tectonic, open the
  generated PDF externally, and report source/output paths in GUI status.
  Embedded GUI/TUI PDF preview remains open.
- 2026-06-02: Moved runtime GUI `.tex` opens into an async Iced task with a
  `Compiling ...` status and completion message.
- 2026-06-02: Updated GUI startup to return an initial Iced task for `.tex`
  launch arguments, so `paperview-gui file.tex` no longer compiles during state
  construction.
- 2026-06-02: Moved the default generated `.tex` PDF path from beside the
  source file to `.paperview/tex/<name>.pdf` under the source file's directory.
  Explicit output paths remain supported in the core API.
- 2026-06-02: Added `paperview-tui tex clean <file.tex|dir>` for removing a
  single managed PDF artifact or a directory-level `.paperview/tex/` cache.
- 2026-06-02: Added a GUI `Open PDF` header action after successful `.tex`
  compilation so users can reopen the managed PDF artifact from the compiled
  status.
- 2026-06-02: Added a GUI `Clean PDF` header action after successful `.tex`
  compilation so users can remove the managed PDF artifact from the compiled
  status without switching to the TUI.
