# TUI TOC Scroll Sync

## Goal and Scope

Bring Scroll Synchronization parity to the Ratatui reader by highlighting the
active table-of-contents section as the user scrolls.

This plan covers:

- Track rendered line anchors for parsed document blocks.
- Derive the active heading from the current TUI scroll offset.
- Render the active TOC item with a visible marker and accent style.
- Preserve the active highlight across live reload and bounded scroll changes.
- Update Scroll Synchronization feature, README, tracker, and plan docs.

Out of scope:

- TUI click-to-scroll support.
- TUI keyboard TOC selection/jump mode.
- Exact wrapped terminal line geometry.
- Split-pane scroll synchronization.

## Affected Paths

- `crates/paperview-tui/src/app.rs`
- `crates/paperview-tui/src/render.rs`
- `docs/features/scroll-synchronization.md`
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

Scroll the TUI reader and confirm the active TOC marker follows the visible
section.

## Progress Notes

- Added rendered line anchors and active TOC highlighting for the Ratatui reader.
- Completed with active-section marker rendering, focused TUI tests, docs
  updates, and TUI launch smoke verification.
