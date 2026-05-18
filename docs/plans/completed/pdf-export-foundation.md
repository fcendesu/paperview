# PDF Export Foundation

## Goal

Reserve the PDF export command contract and move export format handling into
core before adding a real PDF backend.

## Scope

- `crates/paperview-core/src/export.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-tui/src/main.rs`
- README, task tracker, architecture notes, and feature docs

## Implementation Steps

1. Add a shared `ExportFormat` enum for `html` and `pdf`.
2. Add an `ExportArtifact` type for completed export backend output.
3. Add explicit export errors for unavailable backends.
4. Route existing HTML generation through `export_document`.
5. Parse `paperview-tui export <file> --to html|pdf` through core format
   parsing.
6. Document PDF as recognized but backend-unavailable.

## Outcome

PaperView now has a core-owned export contract. HTML export still writes an
`.html` artifact beside the source file, while `--to pdf` fails clearly with:

```text
PDF export is not available yet
```

This keeps the command surface stable for the future PDF backend without adding
a heavyweight dependency prematurely.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core export
cargo test -p paperview-tui export
cargo run -p paperview-tui -- export docs/PRD.md --to html
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
