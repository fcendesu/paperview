# In-Document Search Foundation

## Goal

Add a shared in-document text search API and expose it in the TUI reader as a first keyboard-driven search workflow.

## Scope

- `crates/paperview-core/src/search.rs`
- `crates/paperview-core/src/document.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-tui/src/app.rs`
- README and relevant project docs

## Implementation Steps

1. Added a case-insensitive line-based search API in core.
2. Exposed document search through `Document::search`.
3. Added TUI search mode with `/`, query entry, `Enter`, `n`, and `N`.
4. Refreshed search results after reload and bounded the selected match.
5. Added core and TUI tests.
6. Updated feature specs and trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core search
cargo test -p paperview-tui search
```

Full workspace checks were also run before completion.

## Outcome

In-document source search is available as a shared core capability and a TUI reader workflow. GUI search UI, rendered-match highlighting, and ripgrep-backed workspace search remain deferred.
