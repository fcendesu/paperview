# HTML Export Command

## Goal

Add a first headless HTML export command using the shared parsed document model.

## Scope

- `crates/paperview-core/src/export.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-tui/src/main.rs`
- README, task tracker, architecture notes, and feature docs

## Implementation Steps

1. Add a core HTML exporter for the current block model.
2. Escape text and attributes safely.
3. Render headings, paragraphs, blockquotes, code, diagrams, math, images,
   tables, lists, task lists, and rules.
4. Add `paperview-tui export <file> --to html`.
5. Add focused core and CLI tests.
6. Update docs and trackers.

## Outcome

PaperView now exports parsed documents to standalone HTML through
`paperview-core::export_html` and exposes the command as:

```sh
cargo run -p paperview-tui -- export docs/PRD.md --to html
```

The first backend writes the derived `.html` path beside the source file. PDF
export, rendered math and Mermaid assets, exported heading anchors, syntax
highlighting, and template customization remain deferred.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core export
cargo test -p paperview-tui export
cargo run -p paperview-tui -- export docs/PRD.md --to html
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
