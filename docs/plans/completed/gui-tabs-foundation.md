# GUI Tabs Foundation

## Goal and Scope

Introduce the first tabbed-document model and GUI tab activation behavior.

This plan covered:

- Add a core open-document collection for tab state.
- Refactor GUI state from a single optional document to open documents plus an active tab.
- Open launch/history/drop files into tabs, activating an existing tab when the path is already open.
- Render clickable GUI tabs for all open documents.
- Keep live reload scoped to the active tab.
- Keep Zen Mode behavior intact.
- Update feature, architecture, design, and tracker docs.

Out of scope:

- Closing tabs.
- Reordering tabs.
- Opening multiple launch arguments.
- Split-view integration.
- TUI tabs.

## Affected Paths

- `crates/paperview-core/src/open_documents.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/tabs.md`
- `docs/features/INDEX.md`
- `docs/arch/INDEX.md`
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

Then open a second document from history or drag/drop and confirm tab activation works.

## Final Outcome

- `paperview-core::OpenDocuments` owns shared tab state.
- GUI opens launch/history/drop files into tabs and activates existing tabs by path.
- The tab bar renders all open documents as clickable tab buttons.
- Live reload continues to watch only the active tab for this foundation slice.
