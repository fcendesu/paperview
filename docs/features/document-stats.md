# Document Stats

## Product Behavior

PaperView can print a compact document statistics report without launching a UI:

```sh
cargo run -p paperview-tui -- stats docs/PRD.md
```

It can also emit the same stats as JSON for scripts and automation:

```sh
cargo run -p paperview-tui -- stats docs/PRD.md --json
```

The report includes:

- File path
- Document title
- Word count
- Line count
- Character count
- Heading count
- Estimated reading time
- Heading structure

The JSON report uses these keys: `file`, `title`, `words`, `lines`, `characters`, `headings`, `estimated_reading_minutes`, and `heading_structure`.

## Implementation Notes

- `paperview-core::stats::document_stats` builds the shared stats model.
- `Document::stats` exposes stats for already-loaded documents.
- Word count is a lightweight alphanumeric token count from source text.
- Estimated reading time uses a 200 words-per-minute baseline and rounds up.
- The TUI binary handles `stats <file>` and `stats <file> --json` as headless commands and exits without initializing Ratatui.
- JSON formatting lives in the TUI crate and is built from `Document::stats`.

## Decisions And Gaps

- Stats currently use source text rather than a fully rendered/plain-text AST projection.
- Heading structure comes from the shared parser TOC.
- Richer metadata is deferred.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core stats
cargo test -p paperview-tui stats
```

Run workspace checks before finishing stats changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
