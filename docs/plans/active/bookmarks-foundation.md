# Bookmarks Foundation Plan

## Goal

Add the first durable bookmark slice: shared core bookmark persistence plus
headless TUI commands to add, list, remove, and prune bookmarks.

## Scope

- Add a core `Bookmark`, `Bookmarks`, and `BookmarkStore` using the existing
  history/config persistence style.
- Store document path, title, optional heading anchor, optional source line, and
  optional scroll progress.
- Add TUI commands for bookmark list/add/remove/prune.
- Update the bookmark feature spec, tracker, and README.

## Out Of Scope

- GUI sidebar integration.
- TUI in-reader keyboard shortcuts and jump navigation.
- Per-workspace bookmark stores.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core bookmark`
- `cargo test -p paperview-tui bookmark`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

## Progress

- 2026-06-02: Plan opened for core bookmark persistence and headless TUI
  command foundation.
- 2026-06-02: Added core `Bookmark`, `Bookmarks`, and `BookmarkStore`
  persistence plus headless TUI `bookmark list/add/remove/prune` commands.
- 2026-06-02: Added `paperview-tui bookmark interactive`, an interactive
  bookmark picker that opens the selected bookmark in the reader near its
  stored source line when available.
- 2026-06-02: Added TUI reader shortcut `m` to bookmark the current
  file-backed document location with source-line and active-heading metadata.
- 2026-06-02: Added GUI left-sidebar bookmark visibility and click-to-open
  behavior, with startup pruning for missing bookmarked paths.
- 2026-06-02: Added GUI header bookmark creation for the active file-backed
  document, including immediate sidebar state refresh.
