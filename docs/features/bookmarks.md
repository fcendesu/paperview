# Bookmarks

## Product Behavior

Bookmarks are planned but not implemented yet.

The intended feature is a viewer-first way to save useful reading locations
inside documents without turning PaperView into a general notes database.

The first slice should support:

- Bookmark the current document location.
- Store the document path, title, optional heading anchor, and source-line or
  scroll position metadata.
- Show saved bookmarks in the GUI navigation/sidebar area.
- Expose keyboard-driven TUI bookmark actions and a bookmark list.
- Persist bookmarks across launches.

Bookmarks are distinct from:

- Recent-file history, which records documents opened automatically.
- PDF outline/bookmark entries generated during PDF export.
- Browser-style external bookmarks.

## Implementation Notes

- Core should own a shared `Bookmark` model and `BookmarkStore`, similar to the
  existing history/config store pattern.
- GUI and TUI frontends should remain thin: they should create, list, navigate
  to, and remove bookmarks through core APIs.
- Bookmark targets should prefer stable document identity plus heading/source
  metadata. Exact pixel scroll offsets are frontend-specific and should not be
  the only stored target.
- Bookmark persistence should prune or mark missing document paths gracefully.

## Decisions And Gaps

- Decide whether bookmarks should be global, per-workspace, or both.
- Decide whether a bookmark is tied to a heading, a source line, scroll
  progress, or a combination.
- Decide keyboard shortcuts for GUI and TUI without colliding with existing
  navigation, tabs, search, editing, presentation, and split-view controls.
- Decide whether the GUI history rail and bookmarks share a panel or use
  separate tabs.

## Verification Expectations

The first implementation should include:

```sh
cargo fmt --all
cargo test -p paperview-core bookmark
cargo test -p paperview-gui bookmark
cargo test -p paperview-tui bookmark
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
