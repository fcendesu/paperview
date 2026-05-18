# HTML Export

## Product Behavior

PaperView can export a Markdown document to standalone HTML without launching a UI:

```sh
cargo run -p paperview-tui -- export docs/PRD.md --to html
```

The command writes an `.html` file beside the source document and prints the
output path. For example, `docs/PRD.md` exports to `docs/PRD.html`.

## Implementation Notes

- `paperview-core::export_html` renders from the shared parsed `Document` model.
- `paperview-core::export_document` routes the shared `ExportFormat::Html`
  backend into an `ExportArtifact` that the CLI can write.
- The exporter escapes text and HTML attributes before writing user content.
- Exported headings include duplicate-safe `id` attributes using the same
  slug policy as the shared table of contents.
- Current output covers headings, paragraphs, blockquotes, code blocks,
  Mermaid/source diagram blocks, image blocks, tables, lists, task-list
  checkboxes, math blocks, and rules.
- `paperview-tui export <file> --to html` loads the document, writes the HTML
  beside the source file, prints the output path, and exits without initializing
  Ratatui.

## Decisions And Gaps

- HTML is the first export backend; PDF export is recognized by the command
  layer but remains backend-unavailable.
- Math and Mermaid export are source-preserving rather than fully rendered.
- External asset bundling, syntax highlighting, and template customization are
  deferred.
- Existing files with the derived `.html` path are overwritten.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core export
cargo test -p paperview-tui export
cargo run -p paperview-tui -- export docs/PRD.md --to html
```

Remove generated smoke-test HTML when it is not intended as a repository
artifact.

Run workspace checks before finishing export changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
