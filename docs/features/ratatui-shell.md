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
  - `o` opens a prompt for a pasted or typed file path and opens the document as a tab.

Launching without a file opens a recent-files dashboard backed by persisted history.

## Implementation Notes

- Runtime shell lives in `crates/paperview-tui/src/app.rs`.
- Plain text conversion helpers remain in `crates/paperview-tui/src/render.rs` for tests and widget input.
- The TUI uses `ratatui` with the Crossterm backend and `crossterm` for keyboard events.
- The app restores the terminal after the run loop exits.
- The dashboard shares the same terminal setup and opens selected recent files into the reader shell.
- The reader shell supports an open-path input mode for terminal-friendly file opening when native
  drag-and-drop events are unavailable.

## Open Decisions

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
