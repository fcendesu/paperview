# GUI Tab Close

## Goal and Scope

Add close controls to GUI tabs and core tab-state semantics.

This plan covered:

- Add `OpenDocuments::close`.
- Close GUI tabs from the tab bar.
- Activate a neighboring tab when closing the active tab.
- Return to the empty state when the last tab closes.
- Keep live reload targeting the new active tab.
- Update tabs docs and tracker notes.

Out of scope:

- Tab reordering.
- Dirty-document prompts.
- Keyboard shortcuts for closing tabs.
- TUI tabs.

## Affected Paths

- `crates/paperview-core/src/open_documents.rs`
- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/tabs.md`
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

Open another document, close tabs, and confirm the active document changes or the empty state appears.

## Final Outcome

- `OpenDocuments::close` removes tabs and updates the active index.
- GUI tabs now include a compact close control.
- Closing the active tab selects the next available tab; closing the final tab returns to the empty state.
