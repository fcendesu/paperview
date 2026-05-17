# GUI TOC Scroll Sync

## Goal and Scope

Add the first GUI scroll synchronization behavior by highlighting the table of
contents entry that corresponds to the reader's current scroll position.

This plan covers:

- Observe active reader scroll events through Iced's scrollable viewport.
- Map scroll progress onto heading block positions from the parsed document.
- Store the active heading in GUI state.
- Highlight the active table-of-contents item.
- Reset or recompute the active heading when the active document changes.
- Update feature, design, README, tracker, and plan docs.

Out of scope:

- Pixel-perfect heading geometry.
- Click-to-scroll TOC navigation.
- Split-pane scroll synchronization.
- Independent split-pane scroll persistence.
- TUI scroll synchronization.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/navigation.rs`
- `crates/paperview-gui/src/reader.rs`
- `docs/features/scroll-synchronization.md`
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

Scroll the GUI reader and confirm the right-hand TOC highlight moves through the
document sections.

## Progress Notes

- Added active-reader scroll observation and TOC highlight state.
- Completed with scroll progress mapping, navigation highlight rendering,
  focused tests, documentation updates, and GUI smoke verification.
