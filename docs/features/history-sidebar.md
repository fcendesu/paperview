# History Sidebar

## Product Behavior

PaperView has a shared recent-file model, a GUI History sidebar, and a TUI recent-files dashboard. When either frontend launches with a document path, the loaded document is recorded and moved to the top of the history list.

The current history surfaces support opening recent files:

- Shows the "History" heading.
- Shows persisted recent document titles and paths.
- Uses the dark shell visual treatment from the design spec.
- Lets the GUI sidebar reopen a recent document by clicking the history item.
- Lets the TUI dashboard select recent files with `j`/`k` and open them with Enter.

Grouping by date, richer metadata, and stale-entry management are deferred.

## Implementation Notes

- Core history metadata lives in `paperview-core/src/history.rs`.
- `FileEntry` stores document title and path.
- `History` stores entries in newest-first order and de-duplicates by path.
- `HistoryStore` reads and writes history as TOML.
- Missing history files load as empty history.
- The default history path is platform-aware and can be overridden with `PAPERVIEW_HISTORY_PATH` for tests and smoke runs.
- GUI rendering lives in `crates/paperview-gui/src/history.rs`.
- GUI state loads persisted history on startup, records the initially loaded document, and saves the updated history.
- GUI history clicks route through `Message::OpenHistory`, open the selected path with `Document::open`, record and persist successful opens, and show failures in the window status.
- TUI dashboard rendering and selection live in `crates/paperview-tui/src/app.rs`.
- `paperview-tui` without a file opens the recent-files dashboard.

## Open Decisions

- Date grouping should be introduced once entries have durable timestamps.
- Deleting or pruning stale history entries is deferred.

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
PAPERVIEW_HISTORY_PATH=<temp-file> cargo run -p paperview-tui
```
