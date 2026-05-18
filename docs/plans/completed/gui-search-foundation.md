# GUI Search Foundation

## Goal

Expose the shared in-document search API in the GUI header with query entry and match navigation.

## Scope

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/in-document-search.md`
- `docs/TASKS.md`
- README and design notes

## Implementation Steps

1. Added GUI search state for query, matches, and selected match.
2. Added a compact header search input plus previous/next controls.
3. Refreshed search results when the query, active document, or reloaded document changes.
4. Scrolled the active reader to the selected match using source-line progress.
5. Added GUI state tests.
6. Updated docs and trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-gui search
```

Full workspace checks were also run before completion.

## Outcome

GUI in-document search is available in the header. Match highlighting and exact rendered-line geometry remain deferred.
