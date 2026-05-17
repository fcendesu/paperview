# PaperView Execution Plans

This directory stores execution plans for complex implementation work.

Create a plan before work that spans multiple crates, changes architecture, introduces a major feature, or carries notable product/design risk.

## Layout

- `active/` - plans for work currently in progress.
- `completed/` - retained records of finished plans.
- `tech-debt-tracker.md` - known shortcuts, deferred cleanup, and follow-up work.

## Completed Plans

- [GUI Iced Shell](completed/gui-iced-shell.md) - first native GUI window with optional file loading and simple reader widgets.
- [History Sidebar Foundation](completed/history-sidebar-foundation.md) - shared recent-file model and first GUI history rail.
- [History Persistence](completed/history-persistence.md) - TOML-backed recent-file storage loaded and saved by the GUI.
- [GUI History Open](completed/gui-history-open.md) - clickable GUI history entries that reopen and persist recent documents.
- [GUI Drag And Drop](completed/gui-drag-and-drop.md) - native GUI file-drop opening.
- [GUI Zen Mode](completed/gui-zen-mode.md) - focused GUI reader layout.
- [GUI Tabs Foundation](completed/gui-tabs-foundation.md) - shared open-document model and GUI tab activation.
- [GUI Tab Close](completed/gui-tab-close.md) - close controls and active-tab fallback behavior.
- [GUI Multi-File Drop](completed/gui-multi-file-drop.md) - multi-file drops open supported files into tabs.
- [GUI Split View Foundation](completed/gui-split-view-foundation.md) - side-by-side GUI reader panes for two open tabs.
- [GUI Split View Controls](completed/gui-split-view-controls.md) - visible Split View toggle and secondary tab selector.
- [GUI Split View Resizing](completed/gui-split-view-resizing.md) - keyboard resizing for proportional split panes.
- [GUI TOC Scroll Sync](completed/gui-toc-scroll-sync.md) - active-reader scroll progress highlights the current TOC section.
- [GUI TOC Click Navigation](completed/gui-toc-click-navigation.md) - clickable TOC entries jump the active reader.
- [GUI Reader Scroll Geometry](completed/gui-reader-scroll-geometry.md) - reader-aware estimated heading anchors for TOC sync.
- [Live Reload Foundation](completed/live-reload-foundation.md) - core file watcher and GUI active-document reload.
- [TUI Live Reload](completed/tui-live-reload.md) - Ratatui active-document reload using the core watcher.
- [TUI TOC Scroll Sync](completed/tui-toc-scroll-sync.md) - Ratatui reader highlights the active TOC section while scrolling.
- [Ratatui Shell](completed/ratatui-shell.md) - first interactive terminal reader shell.
- [TUI History Dashboard](completed/tui-history-dashboard.md) - no-file Ratatui recent-files dashboard with selection and open.

Each plan should include:

- Goal and scope.
- Affected source paths and docs.
- Implementation steps.
- Verification plan.
- Progress notes or final outcome.

When a plan is completed, keep it as a record unless it becomes misleading. Update or remove stale plans immediately.
