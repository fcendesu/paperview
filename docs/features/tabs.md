# Tabs

## Product Behavior

PaperView can keep multiple documents open and switch between them using tabs.

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
- Launching the TUI with multiple file paths opens them as tabs.
- The TUI header shows a compact tab row with the active tab highlighted.
- `[` and `]` switch to the previous and next TUI tab.
- `x` closes the active TUI tab.
- Closing a TUI tab activates the next available neighbor; closing the final
  tab exits the reader.
- The TUI reader, table of contents, live reload target, and search results
  follow the active tab.

Tab reordering and split-view integration are deferred.

## Implementation Notes

- Shared tab state lives in `paperview-core/src/open_documents.rs`.
- `OpenDocuments` stores documents plus an active index and handles open-or-activate and close behavior.
- GUI state owns `OpenDocuments` instead of a single optional `Document`.
- GUI tab rendering lives in `crates/paperview-gui/src/app.rs`.
- Tab styling lives in `crates/paperview-gui/src/theme.rs`.
- TUI state owns `OpenDocuments` instead of a single `Document`.
- TUI tab rendering, keyboard switching, and close behavior live in
  `crates/paperview-tui/src/app.rs`.
- `paperview-tui [file ...]` opens multiple documents into the TUI tab set.
- Live reload remains scoped to the active tab for this slice.

## Open Decisions

- Multi-file drag/drop uses tabs; folder drops remain deferred.
- TUI tab reordering is deferred.
- Split view should build on top of the same core document collection instead of forking document state.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo test -p paperview-tui tab
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For visual smoke testing:

```sh
cargo run -p paperview-gui -- docs/PRD.md
cargo run -p paperview-tui -- docs/PRD.md README.md
```

Open another document from History or drag-and-drop, then click between tabs and confirm the reader, title, and TOC update.
In the TUI smoke test, use `[` and `]` to switch tabs and confirm the reader,
title, TOC, and live header tab highlight follow the active document. Use `x`
to close a tab and confirm the neighbor becomes active.
