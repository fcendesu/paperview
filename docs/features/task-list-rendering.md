# Task List Rendering

## Product Behavior

PaperView renders Markdown task-list items in both reader frontends:

- Checked items such as `- [x] Done`
- Unchecked items such as `- [ ] Todo`
- Mixed normal list items and task-list items in the same Markdown list

The current behavior is read-only. Task checkboxes are visual markers, not editable controls.

## Implementation Notes

- `paperview-core::parser::ListItem` stores each list item's inline content and optional task checkbox state.
- `parse_markdown` captures `pulldown-cmark` task-list marker events into `ListItem::checked`.
- Normal ordered and unordered lists continue to render from the same `Block::List` model with `checked: None`.
- The GUI renders task items with checkbox glyphs next to rich inline text.
- The TUI renders task items as Markdown-shaped `- [x]` and `- [ ]` lines.

## Decisions And Gaps

- Task list rendering is intentionally read-only for the MVP viewer surface.
- Interactive task toggles are deferred until PaperView has an edit/writeback model.
- Nested list structure is still flattened by the current basic list model.

## Verification Expectations

Run parser and TUI focused checks with:

```sh
cargo test -p paperview-core task
cargo test -p paperview-tui task
```

Run workspace checks before finishing task-list parser or renderer changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

