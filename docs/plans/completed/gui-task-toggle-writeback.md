# GUI Task Toggle Writeback

## Goal

Let the GUI toggle Markdown task-list checkboxes directly in file-backed
documents while keeping the shared parser model and TUI rendering predictable.

## Scope

- `crates/paperview-core/src/parser/mod.rs`
- `crates/paperview-core/src/document.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/task-list-rendering.md`
- `docs/arch/INDEX.md`
- `docs/design/INDEX.md`
- `docs/TASKS.md`
- `README.md`

## Outcome

- Parsed task-list items now include source line metadata when their checkbox
  marker can be matched to the original Markdown source.
- Core exposes `toggle_task_line_source` for toggling a single task marker
  without changing surrounding text.
- The GUI renders file-backed task markers as clickable controls, writes the
  updated source to disk, and reloads the active document.
- The TUI remains read-only for this slice.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core task`
- `cargo test -p paperview-gui task`
- `cargo test -p paperview-tui task`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
