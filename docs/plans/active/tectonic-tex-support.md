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
