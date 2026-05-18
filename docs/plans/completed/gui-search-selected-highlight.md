# GUI Search Selected Highlight

## Goal

Make the selected GUI in-document search match visually distinct from other
matches in the rendered reader.

## Scope

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-gui/src/theme.rs`
- Search feature, design, and task-tracker docs

## Implementation Steps

1. Thread the selected search result's source line from `PaperView` into the
   active reader.
2. Add a reader search context that distinguishes normal matches from the
   selected match's rendered block.
3. Add active search highlight colors.
4. Extend focused reader tests for normal highlights, active highlights, and
   source-line-to-rendered-block matching.
5. Update feature and design documentation.

## Outcome

The GUI still highlights all rendered query matches, and now gives matches in
the selected result's rendered block a stronger active highlight. This keeps
the current line-based search model while making previous/next navigation
easier to follow visually.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-gui search
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
