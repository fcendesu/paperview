# Table of Contents

## Product Behavior

PaperView derives a table of contents from Markdown headings. The GUI shows the current document's headings in a right-side "On this page" navigation rail, and the TUI shows the headings in a right-side Ratatui panel.

The first TOC slice supports:

- Heading title text.
- Heading depth from H1 through H6.
- Stable slug generation for duplicate headings.
- Source block index for future scroll synchronization.
- Empty-state text when a document has no headings.

The GUI TOC supports click-to-scroll navigation. The TUI TOC supports keyboard
focus with `Tab`, selection with `j` / `k` or arrow keys, and jump-to-heading
with `Enter`.

## Implementation Notes

- Core TOC extraction lives on `paperview_core::parser::ParsedDocument::toc`.
- TOC records use `TocItem`, which includes `level`, `title`, `slug`, and `block_index`.
- GUI rendering lives in `crates/paperview-gui/src/navigation.rs`.
- The GUI reader and TOC are composed side by side only when a document is loaded.
- TUI rendering uses `crates/paperview-tui/src/app.rs` for the interactive shell and `crates/paperview-tui/src/render.rs` for text conversion helpers.

## Open Decisions

- Slugs are internal metadata for now; future exported HTML may need a shared anchor policy.
- Mouse-based TOC navigation is deferred.
- Exact wrapped terminal line geometry remains separate from static TOC rendering.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For visual smoke testing:

```sh
cargo run -p paperview-gui -- docs/PRD.md
```
