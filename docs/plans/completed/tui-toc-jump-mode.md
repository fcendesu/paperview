# TUI TOC Jump Mode

## Goal and Scope

Let terminal readers navigate documents from the right-hand table of contents.

This plan covers:

- Add a reader/TOC focus mode in the Ratatui reader.
- Use `Tab` to toggle focus between reader scrolling and TOC selection.
- Use `j/k` or arrow keys to move through TOC entries while focused.
- Use `Enter` to jump the reader to the selected heading.
- Keep active-section highlighting while the reader scrolls.
- Update Scroll Synchronization, Table of Contents, README, tracker, and plan docs.

Out of scope:

- Mouse support in the TUI.
- Persisted TOC focus/selection state.
- Split-pane scroll synchronization.
- Exact wrapped terminal line geometry.

## Affected Paths

- `crates/paperview-tui/src/app.rs`
- `crates/paperview-tui/src/render.rs`
- `docs/features/scroll-synchronization.md`
- `docs/features/table-of-contents.md`
- `docs/TASKS.md`
- `README.md`

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Smoke test:

```sh
cargo run -p paperview-tui -- docs/PRD.md
```

Press `Tab`, move through TOC entries, press `Enter`, and confirm the reader
jumps to the selected heading.

## Progress Notes

- Added TUI reader/TOC focus, bounded TOC selection, and jump-to-heading
  behavior.
- Completed with focused interaction tests, selected-TOC rendering, docs
  updates, and TTY smoke verification.
