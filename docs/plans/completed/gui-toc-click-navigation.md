# GUI TOC Click Navigation

## Goal and Scope

Make table-of-contents entries clickable in the GUI so readers can jump through
documents from the navigation sidebar.

This plan covers:

- Add a stable ID for the active reader scrollable.
- Add a `TocSelected` message and return an Iced task that snaps the reader.
- Map heading block indices to relative scroll offsets.
- Render TOC items as clickable rows with active styling.
- Update Scroll Synchronization feature, design, README, tracker, and plan docs.

Out of scope:

- Pixel-perfect scroll positioning for each heading.
- TUI click/key navigation parity.
- Split-pane secondary TOC navigation.
- Persisted scroll positions.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/navigation.rs`
- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/scroll-synchronization.md`
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

Click TOC entries and confirm the active reader jumps and highlights the selected
section.

## Progress Notes

- Added clickable TOC rows and active-reader scroll tasks.
- Completed with a stable active-reader scrollable ID, TOC selection message,
  relative scroll mapping tests, documentation updates, and GUI smoke
  verification.
