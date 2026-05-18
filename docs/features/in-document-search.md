# In-Document Search

## Product Behavior

PaperView has a shared in-document search foundation plus GUI and TUI search workflows.

Current GUI behavior:

- The header includes a search field for the active document.
- Typing a query finds case-insensitive source-line matches.
- Previous and next controls cycle through matches.
- Selecting a match scrolls the active reader near the matching source line.
- Matching rendered reader text is highlighted in the active document.

Current TUI controls:

- `/` enters search mode.
- Type a query and press `Enter` to jump to the first match.
- `n` jumps to the next match.
- `N` jumps to the previous match.
- `Esc` cancels search entry.
- Matching TUI lines are highlighted, with the selected match emphasized.

Search is case-insensitive and line-based. It searches the source document text and scrolls the TUI reader to the matching source line.

## Implementation Notes

- `paperview-core::search::search_lines` returns line index, column, and source line text for matches.
- `Document::search` exposes source search without requiring frontends to inspect document internals.
- The GUI stores query, result list, and selected match state in `PaperView` and uses the active reader scroll operation to jump matches.
- GUI reader rendering splits rich inline spans around query matches and applies search highlight styling while preserving inline formatting and links.
- The TUI reader keeps search query, result list, and selected match state in `ReaderApp`.
- TUI reader rendering styles matched lines and gives the selected match a stronger highlighted span.
- Search results are refreshed after live reload so the selected result remains bounded to the reloaded document.

## Decisions And Gaps

- Interactive workspace-search result selection is deferred; the current headless workspace command prints results.
- Selected-match emphasis in the GUI is deferred; the current GUI highlights matching rendered text occurrences without distinguishing the active match.
- Source-line search can drift from rendered-line geometry for wrapped paragraphs and complex Markdown blocks.

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
