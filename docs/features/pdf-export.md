# PDF Export

## Product Behavior

PaperView reserves the headless PDF export command shape:

```sh
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
```

The command currently returns a clear unavailable-backend error instead of
silently ignoring PDF or treating it as an unknown format.

## Implementation Notes

- `paperview-core::ExportFormat` owns supported export format parsing for
  `html` and `pdf`.
- `paperview-core::export_document` returns an `ExportArtifact` for completed
  backends and an `ExportError` for unavailable ones.
- HTML export is implemented through the shared export path.
- PDF export is represented as `ExportError::PdfUnavailable`.
- The TUI command parses `--to html|pdf`, asks core for an artifact, writes
  successful output beside the source document, and prints the output path.

## Decisions And Gaps

- PDF generation is intentionally not implemented yet.
- The first real backend should reuse the HTML export styling when possible so
  HTML and PDF remain visually aligned.
- Candidate future paths include a Rust-native PDF renderer or HTML-to-PDF
  pipeline; dependency weight, offline behavior, and cross-platform packaging
  should be decided before adding the backend.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core export
cargo test -p paperview-tui export
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
```

The PDF smoke command should fail with `PDF export is not available yet` until a
real backend lands.

Run workspace checks before finishing export changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
