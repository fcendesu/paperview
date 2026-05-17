# Tabs

## Product Behavior

PaperView GUI can keep multiple documents open and switch between them using a tab bar.

Current behavior:

- Launching the GUI with a file opens one active tab.
- Opening a file from History or drag-and-drop adds a tab when the path is not already open.
- Dropping multiple supported files opens each one as a tab.
- Opening a path that is already open refreshes that tab and activates it.
- Clicking a tab activates that document.
- Clicking a tab close control removes that tab.
- Closing the active tab activates the next available neighbor; closing the final tab returns to the empty state.
- The window title, reader, table of contents, status, and live reload target follow the active tab.
- Zen Mode hides the tab bar but preserves the open tab set and active tab.

Tab reordering, multi-file launch arguments, and split-view integration are deferred.

## Implementation Notes

- Shared tab state lives in `paperview-core/src/open_documents.rs`.
- `OpenDocuments` stores documents plus an active index and handles open-or-activate and close behavior.
- GUI state owns `OpenDocuments` instead of a single optional `Document`.
- GUI tab rendering lives in `crates/paperview-gui/src/app.rs`.
- Tab styling lives in `crates/paperview-gui/src/theme.rs`.
- Live reload remains scoped to the active tab for this slice.

## Open Decisions

- Multi-file drag/drop uses tabs; folder drops remain deferred.
- TUI tabs are deferred; the TUI remains a single-reader workflow for now.
- Split view should build on top of the same core document collection instead of forking document state.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For visual smoke testing:

```sh
cargo run -p paperview-gui -- docs/PRD.md
```

Open another document from History or drag-and-drop, then click between tabs and confirm the reader, title, and TOC update.
