# TUI Split View Controls

## Goal

Let terminal users change the secondary Split View pane without changing the
active reader tab.

## Completed

- Added `{` / `}` key handling for cycling the secondary pane while Split View
  is enabled.
- Kept `[` / `]` focused on active-tab navigation.
- Added header help text for the side-pane controls.
- Added tests for side-pane wraparound and the disabled-split no-op path.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui split`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
