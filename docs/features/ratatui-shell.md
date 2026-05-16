# Ratatui Shell

## Product Behavior

PaperView now launches `paperview-tui <file>` into an interactive Ratatui terminal shell.

The first shell supports:

- Full-screen alternate-screen terminal UI.
- Scrollable document reader.
- Right-side table of contents panel.
- Keyboard controls:
  - `q` or Esc quits.
  - `j` or Down scrolls down.
  - `k` or Up scrolls up.
  - `g` jumps to the top.
  - `G` jumps to the bottom.

Launching without a file still prints a short prompt rather than opening a dashboard. The dashboard is deferred until recent-file navigation is interactive.

## Implementation Notes

- Runtime shell lives in `crates/paperview-tui/src/app.rs`.
- Plain text conversion helpers remain in `crates/paperview-tui/src/render.rs` for tests and widget input.
- The TUI uses `ratatui` with the Crossterm backend and `crossterm` for keyboard events.
- The app restores the terminal after the run loop exits.

## Open Decisions

- TUI history dashboard is still open.
- Mouse support is still open.
- TOC selection and active-section synchronization are still open.
- The current scroll limit is line-count based; future viewport-aware scroll state should account for wrapping.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For interactive smoke testing:

```sh
cargo run -p paperview-tui -- docs/PRD.md
```
