# Task List Rendering

## Goal

Preserve Markdown task-list checkbox state in the shared parser model and render checked and unchecked task items in both frontends.

## Scope

- `crates/paperview-core/src/parser/mod.rs`
- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-tui/src/render.rs`
- `docs/features/`
- `docs/TASKS.md`
- README and architecture/design docs touched by the behavior change

## Implementation Steps

1. Added a shared list-item model that stores optional task checkbox state beside inline content.
2. Captured `pulldown-cmark` task-list marker events while parsing list items.
3. Rendered task markers in the GUI and Markdown-shaped task markers in the TUI.
4. Added parser and TUI coverage for checked and unchecked task items.
5. Updated feature records and project trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core task
cargo test -p paperview-tui task
cargo check -p paperview-gui
```

Full workspace checks were also run before completion.

## Outcome

Task-list rendering is available as a read-only viewer feature. Interactive checkbox toggles remain deferred until PaperView has an edit/writeback model.

