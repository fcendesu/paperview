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
- [Live Reload Foundation](completed/live-reload-foundation.md) - core file watcher and GUI active-document reload.
- [Ratatui Shell](completed/ratatui-shell.md) - first interactive terminal reader shell.
- [TUI History Dashboard](completed/tui-history-dashboard.md) - no-file Ratatui recent-files dashboard with selection and open.

Each plan should include:

- Goal and scope.
- Affected source paths and docs.
- Implementation steps.
- Verification plan.
- Progress notes or final outcome.

When a plan is completed, keep it as a record unless it becomes misleading. Update or remove stale plans immediately.
