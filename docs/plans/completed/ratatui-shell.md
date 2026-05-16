# Ratatui TUI Shell

## Goal and Scope

Replace the print-only TUI launch path with a minimal interactive Ratatui shell.

This plan covers:

- Add `ratatui` and `crossterm`.
- Launch `paperview-tui <file>` into an alternate-screen terminal UI.
- Render document text in a scrollable viewport.
- Render a right-side TOC panel.
- Support `q`/Esc quit, `j`/Down scroll down, `k`/Up scroll up, `g` top, and `G` bottom.
- Keep the existing plain renderer available for tests.

Out of scope:

- Mouse support.
- History dashboard.
- Click/keyboard TOC navigation.
- Live reload.

## Affected Paths

- `crates/paperview-tui/Cargo.toml`
- `crates/paperview-tui/src/main.rs`
- `crates/paperview-tui/src/render.rs`
- `crates/paperview-tui/src/app.rs`
- `docs/features/basic-markdown-rendering.md`
- `docs/features/file-opening.md`
- `docs/TASKS.md`

## Implementation Steps

1. Add Ratatui/Crossterm dependencies.
2. Add terminal setup/teardown and event loop.
3. Build scrollable document and TOC widgets from core document data.
4. Update docs and tracker.
5. Run required checks and a short launch smoke.

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p paperview-tui -- docs/PRD.md
```

## Progress Notes

- Started after confirming the previous TUI path was only a plain renderer.
- Added a Ratatui app shell with a scrollable reader, TOC panel, and basic keyboard controls.
- Verified formatting, Clippy, workspace tests, and a PTY smoke run that rendered `docs/PRD.md` and exited on `q`.
