# History Sidebar

## Product Behavior

PaperView has a shared recent-file model and a first GUI History sidebar. When the GUI launches with a document path, the loaded document appears as the active history item in the left rail.

The current sidebar is a scaffold:

- Shows the "History" heading.
- Shows the current loaded document title and path.
- Uses the dark shell visual treatment from the design spec.
- Keeps history display-only.

Persistence, grouping by date, click-to-open behavior, and multi-entry history are deferred.

## Implementation Notes

- Core history metadata lives in `paperview-core/src/history.rs`.
- `FileEntry` stores document title and path.
- `History` stores entries in newest-first order and de-duplicates by path.
- GUI rendering lives in `crates/paperview-gui/src/history.rs`.
- GUI state records the initially loaded document in an in-memory `History`.

## Open Decisions

- Recent-file persistence should be added with the config/history storage slice.
- Date grouping should be introduced once entries have durable timestamps.
- TUI history behavior is deferred until the Ratatui shell becomes stateful.

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
