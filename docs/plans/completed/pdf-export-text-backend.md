# PDF Export Text Backend

## Goal

Turn the reserved `--to pdf` export path into a real artifact without adding a
heavy renderer dependency.

## Scope

- `crates/paperview-core/src/export.rs`
- `crates/paperview-tui/src/main.rs`
- `docs/features/pdf-export.md`
- `docs/features/html-export.md`
- `docs/arch/INDEX.md`
- `docs/TASKS.md`
- `README.md`

## Outcome

- `paperview-core::export_pdf` now writes a PDF 1.4 document using built-in
  Helvetica text.
- `paperview-core::export_document` returns PDF artifacts instead of an
  unavailable-backend error.
- The TUI `export <file> --to pdf` command writes a `.pdf` beside the source
  document and prints the output path.
- The first backend preserves readable text for headings, paragraphs, lists,
  tables, code blocks, math, Mermaid source, rules, and image metadata.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core export`
- `cargo test -p paperview-tui export`
- `cargo run -p paperview-tui -- export docs/PRD.md --to pdf`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
