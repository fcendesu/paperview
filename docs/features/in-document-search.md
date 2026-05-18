# In-Document Search

## Product Behavior

PaperView has a shared in-document search foundation and a first TUI search workflow.

Current TUI controls:

- `/` enters search mode.
- Type a query and press `Enter` to jump to the first match.
- `n` jumps to the next match.
- `N` jumps to the previous match.
- `Esc` cancels search entry.

Search is case-insensitive and line-based. It searches the source document text and scrolls the TUI reader to the matching source line.

## Implementation Notes

- `paperview-core::search::search_lines` returns line index, column, and source line text for matches.
- `Document::search` exposes source search without requiring frontends to inspect document internals.
- The TUI reader keeps search query, result list, and selected match state in `ReaderApp`.
- Search results are refreshed after live reload so the selected result remains bounded to the reloaded document.

## Decisions And Gaps

- GUI search UI is deferred until there is a native input surface in the header or reader chrome.
- Workspace search through `paperview search <query>` is still deferred and should use a separate ripgrep-backed feature.
- TUI match highlighting is deferred; the current behavior jumps to the matching line and reports match position in the header.
- Source-line search can drift from rendered-line geometry for wrapped paragraphs and complex Markdown blocks.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core search
cargo test -p paperview-tui search
```

Run workspace checks before finishing search changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
