# GUI Search Highlighting

## Goal

Make GUI in-document search results visible in the rendered reader text.

## Scope

- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/theme.rs`
- `docs/features/in-document-search.md`
- `docs/TASKS.md`
- README and design notes

## Implementation Steps

1. Passed the active GUI search query into the active reader view.
2. Split rich inline spans around case-insensitive query matches.
3. Applied search highlight styling while preserving inline formatting and links.
4. Covered highlight segmentation with focused reader tests.
5. Updated docs and trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-gui search
```

Full workspace checks were also run before completion.

## Outcome

GUI search now highlights matching rendered text occurrences in the active reader. Selected-match emphasis and exact rendered-line geometry remain deferred.
