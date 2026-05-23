# TUI Split View Resizing

## Goal

Let terminal users resize Split View panes with keyboard controls that match
the GUI's bounded split-ratio model.

## Completed

- Added TUI split width state with a 50/50 default.
- Added `<` / `>` key handling to shrink or grow the primary pane while Split
  View is enabled.
- Clamped the primary pane from 30% to 70% in 10-point steps.
- Reused the stored ratio when rendering terminal split panes.
- Added focused tests for resizing, bounds, and no-op behavior when Split View
  is disabled.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui split_resize`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
