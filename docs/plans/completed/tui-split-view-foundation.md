# TUI Split View Foundation

## Goal

Add a first Ratatui Split View so terminal users can compare the active tab with
another open tab side by side.

## Scope

- `crates/paperview-tui/src/app.rs`
- `docs/features/split-view.md`
- `docs/design/INDEX.md`
- `docs/TASKS.md`
- `README.md`

## Outcome

- The TUI toggles Split View with `\` when multiple tabs are open.
- The active document renders in the left pane and the secondary document
  renders in the right pane.
- Scrolling, search, focus, and TOC highlighting remain owned by the active
  left pane for this foundation slice.
- Split View retargets away from the active tab when tabs change and disables
  itself when no secondary tab remains.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui split`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
