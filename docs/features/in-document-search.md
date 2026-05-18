# In-Document Search

## Product Behavior

PaperView has a shared in-document search foundation plus GUI and TUI search workflows.

Current GUI behavior:

- The header includes a search field for the active document.
- Typing a query finds case-insensitive source-line matches.
- Previous and next controls cycle through matches.
- Selecting a match scrolls the active reader near the matching source line.

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
- The GUI stores query, result list, and selected match state in `PaperView` and uses the active reader scroll operation to jump matches.
- The TUI reader keeps search query, result list, and selected match state in `ReaderApp`.
- Search results are refreshed after live reload so the selected result remains bounded to the reloaded document.

## Decisions And Gaps

- Workspace search through `paperview search <query>` is still deferred and should use a separate ripgrep-backed feature.
- Match highlighting is deferred; the current behavior jumps to the matching line and reports match position.
- Source-line search can drift from rendered-line geometry for wrapped paragraphs and complex Markdown blocks, especially in the GUI.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core search
cargo test -p paperview-gui search
cargo test -p paperview-tui search
```

Run workspace checks before finishing search changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
