# GUI Reader Scroll Geometry

## Goal and Scope

Improve Scroll Synchronization by replacing raw block-count mapping with
reader-aware heading anchors based on estimated rendered block geometry.

This plan covers:

- Move scroll anchor calculations into the GUI reader module.
- Estimate rendered block heights from current reader typography and spacing.
- Use those anchors for both scroll-driven TOC highlighting and TOC click jumps.
- Add focused tests for weighted section mapping.
- Update Scroll Synchronization feature, design, README, tracker, and plan docs.

Out of scope:

- Capturing exact Iced layout rectangles.
- Persisted scroll positions.
- Split-pane secondary scroll synchronization.
- TUI scroll synchronization.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/reader.rs`
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

Scroll and click through a heading-rich document to confirm TOC highlighting and
jump positions feel aligned with visible sections.

## Progress Notes

- Added reader-owned estimated heading anchors for scroll highlighting and TOC
  jumps.
- Completed with weighted reader geometry helpers, scroll/click mapping updates,
  focused tests, documentation updates, and GUI smoke verification.
