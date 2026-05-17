# GUI Split View Resizing

## Goal and Scope

Add the first Split View resizing controls so readers can bias space toward the
primary or secondary document.

This plan covers:

- Store a bounded split ratio in the GUI state.
- Render split panes using proportional widths.
- Add platform command shortcuts to grow or shrink the active pane.
- Add focused tests for ratio bounds and shortcut routing.
- Update Split View feature, design, README, and tracker docs.

Out of scope:

- Mouse dragging for the divider.
- Persisting the split ratio.
- Independent scroll state.
- Scroll synchronization.
- TUI Split View.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `docs/features/split-view.md`
- `docs/design/INDEX.md`
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
cargo run -p paperview-gui -- docs/PRD.md
```

Open a second tab, enable Split View, and verify the resize shortcuts keep the
layout usable.

## Progress Notes

- Added a bounded GUI split ratio and keyboard resize shortcuts.
- Completed with proportional pane rendering, shortcut routing, bounds tests,
  documentation updates, and GUI smoke verification.
