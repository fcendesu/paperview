# Task List Rendering

## Product Behavior

PaperView renders Markdown task-list items in both reader frontends:

- Checked items such as `- [x] Done`
- Unchecked items such as `- [ ] Todo`
- Mixed normal list items and task-list items in the same Markdown list

The GUI can toggle task checkboxes for file-backed documents and writes the
updated marker back to the original Markdown line. The TUI remains read-only and
renders Markdown-shaped task markers.

## Implementation Notes

- `paperview-core::parser::ListItem` stores each list item's inline content and optional task checkbox state.
- `parse_markdown` captures `pulldown-cmark` task-list marker events into `ListItem::checked`.
- Parsed task items also carry a source line index when their checkbox marker
  can be matched back to the source text.
- `paperview-core::toggle_task_line_source` toggles a task marker on a specific
  source line while preserving the rest of the document text.
- Normal ordered and unordered lists continue to render from the same `Block::List` model with `checked: None`.
- The GUI renders file-backed task items with clickable checkbox glyphs next to
  rich inline text.
- The TUI renders task items as Markdown-shaped `- [x]` and `- [ ]` lines.

## Decisions And Gaps

- GUI task toggles are intentionally scoped to file-backed documents.
- TUI task toggles remain deferred.
- Nested list structure is still flattened by the current basic list model.

## Verification Expectations

Run parser and TUI focused checks with:

```sh
cargo test -p paperview-core task
cargo test -p paperview-gui task
cargo test -p paperview-tui task
```

Run workspace checks before finishing task-list parser or renderer changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
