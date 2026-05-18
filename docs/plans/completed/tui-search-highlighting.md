# TUI Search Highlighting

## Goal

Make TUI in-document search results visible in the reader output after a search.

## Scope

- `crates/paperview-tui/src/app.rs`
- `docs/features/in-document-search.md`
- `docs/TASKS.md`
- README and design notes

## Implementation Steps

1. Passed search match state into TUI document-line rendering.
2. Styled matched lines and strongly highlighted the selected match.
3. Preserved existing heading and blockquote styling when no search highlight applies.
4. Added TUI rendering tests for active and inactive search highlights.
5. Updated docs and trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-tui search
```

Full workspace checks were also run before completion.

## Outcome

TUI search now highlights matched reader lines and emphasizes the selected match. GUI match highlighting remains deferred.
