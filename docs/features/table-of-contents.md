# Table of Contents

## Product Behavior

PaperView derives a table of contents from Markdown headings. The GUI shows the current document's headings in a right-side "On this page" navigation rail, and the TUI prints an "On this page" section after the rendered document.

The first TOC slice supports:

- Heading title text.
- Heading depth from H1 through H6.
- Stable slug generation for duplicate headings.
- Source block index for future scroll synchronization.
- Empty-state text when a document has no headings.

The TOC views are display-only for now. Clicking headings, keyboard navigation, and active-section highlighting belong to later interactive navigation and scroll synchronization slices.

## Implementation Notes

- Core TOC extraction lives on `paperview_core::parser::ParsedDocument::toc`.
- TOC records use `TocItem`, which includes `level`, `title`, `slug`, and `block_index`.
- GUI rendering lives in `crates/paperview-gui/src/navigation.rs`.
- The GUI reader and TOC are composed side by side only when a document is loaded.
- TUI rendering lives in `crates/paperview-tui/src/render.rs` and appends the heading list after document content.

## Open Decisions

- Slugs are internal metadata for now; future exported HTML may need a shared anchor policy.
- Interactive TUI TOC behavior is deferred until the Ratatui shell becomes stateful.
- Scroll position tracking and active TOC highlighting remain separate from static TOC rendering.

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
