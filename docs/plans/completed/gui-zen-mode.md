# GUI Zen Mode

## Goal and Scope

Add a focused GUI reading mode that hides secondary chrome.

This plan covered:

- Add GUI layout state for Zen Mode.
- Toggle Zen Mode from the platform command shortcut.
- Hide the tab bar, history sidebar, and table of contents while Zen Mode is active.
- Keep the header, reader, file opening, history, drag-and-drop, and live reload behavior intact.
- Update feature, design, and tracker docs.

Out of scope:

- Auto-hiding header behavior.
- TUI Zen Mode.
- Persistent user preferences.
- A visible toolbar toggle.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `docs/features/zen-mode.md`
- `docs/features/INDEX.md`
- `docs/design/INDEX.md`
- `docs/TASKS.md`

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

Then press Cmd/Ctrl+Shift+F and confirm the reader toggles between full shell and focused layout.

## Final Outcome

- GUI tracks `is_zen` layout state.
- Cmd+Shift+F on macOS and Ctrl+Shift+F elsewhere toggle Zen Mode.
- Zen Mode hides the tab bar, History sidebar, and TOC sidebar while leaving the header and active reader visible.
