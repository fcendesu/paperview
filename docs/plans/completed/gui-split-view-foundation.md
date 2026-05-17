# GUI Split View Foundation

## Goal and Scope

Add the first GUI split-view layout for comparing two open tabs.

This plan covers:

- Add GUI split-view state that points at a secondary open document.
- Toggle split view from a platform command shortcut.
- Render the active tab and secondary tab side by side.
- Keep History and TOC sidebars visible; TOC follows the active tab.
- Keep live reload scoped to the active tab.
- Keep Zen Mode overriding split layout.
- Update feature, design, README, and tracker docs.

Out of scope:

- Independent scroll position persistence.
- Scroll synchronization.
- Drag-to-resize split panes.
- Secondary-tab picker UI.
- TUI split view.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `docs/features/split-view.md`
- `docs/features/INDEX.md`
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

Open a second tab, toggle split view, and confirm two readers render side by side.

## Progress Notes

- Started after multi-file drag/drop into tabs landed.
- Completed with GUI state, shortcut routing, side-by-side rendering, focused
  tests, and documentation updates.
