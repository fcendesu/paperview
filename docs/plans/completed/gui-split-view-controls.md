# GUI Split View Controls

## Goal and Scope

Make the GUI Split View foundation discoverable and controllable without relying
only on the keyboard shortcut.

This plan covers:

- Add a visible Split View toggle in the header.
- Show whether Split View is on or unavailable.
- Allow choosing the secondary pane from non-active tabs while Split View is on.
- Keep the active tab as the primary reader.
- Update Split View feature and design docs.

Out of scope:

- Drag-resizable panes.
- Scroll synchronization.
- Independent scroll state.
- TUI Split View.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
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

Open multiple tabs, toggle Split View from the header, and choose a secondary
tab from the tab bar.

## Progress Notes

- Added a focused plan before changing the GUI controls.
- Completed with a visible header toggle, compact non-active-tab secondary
  selectors, focused state tests, and documentation updates.
