# Bookmarks

## Product Behavior

Bookmarks are partially implemented. The current foundation is a viewer-first
way to persist useful document locations without turning PaperView into a
general notes database.

The first implemented slice supports:

- Store document path, title, optional heading anchor, optional source line, and
  optional scroll progress metadata in a shared core model.
- Persist bookmarks across launches in `bookmarks.toml` under the same
  PaperView data directory pattern as history.
- Override the bookmark store path with `PAPERVIEW_BOOKMARKS_PATH`.
- Add, list, remove, and prune bookmarks through headless TUI commands:
  `bookmark add <file> [--anchor slug|--line number]`, `bookmark list`,
  `bookmark remove <index>`, and `bookmark prune`.
- Open an interactive TUI bookmark picker with `bookmark interactive`; selected
  bookmarks open in the reader near their stored source line when available.

Still planned:

- Bookmark the current in-reader GUI/TUI location with keyboard shortcuts.
- Navigate to heading anchors and scroll-progress targets, not only stored
  source lines.
- Show saved bookmarks in the GUI navigation/sidebar area.

Bookmarks are distinct from:

- Recent-file history, which records documents opened automatically.
- PDF outline/bookmark entries generated during PDF export.
- Browser-style external bookmarks.

## Implementation Notes

- Core owns a shared `Bookmark` model, `Bookmarks` collection, and
  `BookmarkStore`, similar to the existing history/config store pattern.
- GUI and TUI frontends should remain thin: they should create, list, navigate
  to, and remove bookmarks through core APIs.
- Bookmark targets should prefer stable document identity plus heading/source
  metadata. Exact pixel scroll offsets are frontend-specific and should not be
  the only stored target.
- Bookmark persistence can prune missing document paths through the shared
  collection, the `paperview-tui bookmark prune` command, and the interactive
  picker startup path.

## Decisions And Gaps

- Current bookmarks are global by default. Decide whether per-workspace stores
  should be added later.
- Current bookmarks can store heading anchor, source line, and scroll progress
  metadata. Decide how GUI/TUI navigation should choose among those targets.
- Decide keyboard shortcuts for GUI and TUI without colliding with existing
  navigation, tabs, search, editing, presentation, and split-view controls.
- Decide whether the GUI history rail and bookmarks share a panel or use
  separate tabs.

## Verification Expectations

The first implementation should include:

```sh
cargo fmt --all
cargo test -p paperview-core bookmark
cargo test -p paperview-tui bookmark
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
