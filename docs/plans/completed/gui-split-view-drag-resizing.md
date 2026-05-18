# GUI Split View Drag Resizing

## Goal

Add direct mouse resizing to the existing GUI Split View without changing the
shared document model or TUI behavior.

## Scope

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/split-view.md`
- `docs/design/INDEX.md`
- `docs/TASKS.md`
- `README.md`

## Outcome

- Split View now renders a draggable vertical divider between the active and
  secondary panes.
- Dragging the divider updates the same bounded 30% to 70% primary-pane ratio
  used by keyboard resizing.
- Releasing the mouse clears drag state through the normal runtime event
  subscription, even when the pointer is outside the divider.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-gui split`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
