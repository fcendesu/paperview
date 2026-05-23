# PDF Export

## Product Behavior

PaperView can export a Markdown document to a basic text-first PDF without
launching a UI:

```sh
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
```

The command writes a `.pdf` file beside the source document and prints the
output path. For example, `docs/PRD.md` exports to `docs/PRD.pdf`.

## Implementation Notes

- `paperview-core::ExportFormat` owns supported export format parsing for
  `html` and `pdf`.
- `paperview-core::export_document` returns an `ExportArtifact` for HTML and
  PDF.
- HTML export is implemented through the shared export path.
- `paperview-core::export_pdf` writes a dependency-light PDF 1.4 document using
  built-in Helvetica text.
- The first PDF backend preserves headings, paragraphs, lists, tables,
  code/math/diagram source blocks, rules, and image metadata text.
- PDF output uses a larger title treatment, heading sizes and spacing, indented
  lists/source blocks, wrapped text, and vertical-space page breaks.
- The TUI command parses `--to html|pdf`, asks core for an artifact, writes
  successful output beside the source document, and prints the output path.

## Decisions And Gaps

- The PDF backend is still text-first and does not perform rich typography,
  bitmap image embedding, syntax highlighting, Mermaid layout, or LaTeX
  typesetting.
- Future PDF work can replace or augment this writer with a richer renderer if
  dependency weight, offline behavior, and cross-platform packaging stay
  acceptable.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core export
cargo test -p paperview-tui export
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
```

Remove generated smoke-test PDFs when they are not intended as repository
artifacts.

Run workspace checks before finishing export changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
