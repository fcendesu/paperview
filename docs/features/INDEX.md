# PaperView Feature Specs

This directory stores one specification file per major feature. Feature specs are durable implementation records, not chat transcripts.

## Inventory

- [Project Setup](project-setup.md) - workspace layout, crate boundaries, and initial core/frontend shells.
- [File Opening](file-opening.md) - supported document formats and core file loading behavior.
- [Basic Markdown Rendering](basic-markdown-rendering.md) - initial shared Markdown parse model for frontend renderers.
- [Drag And Drop](drag-and-drop.md) - GUI native file-drop opening.
- [Hybrid Theme](hybrid-theme.md) - dark shell, cream reader surface, and GUI visual token contract.
- [History Sidebar](history-sidebar.md) - shared recent-file model and first GUI history rail.
- [Live Reload](live-reload.md) - notify-backed active-document reload foundation.
- [Ratatui Shell](ratatui-shell.md) - first interactive terminal reader shell.
- [Split View](split-view.md) - GUI side-by-side comparison of two open tabs.
- [Table of Contents](table-of-contents.md) - heading-derived navigation metadata and GUI sidebar.
- [Tabs](tabs.md) - shared open-document model and GUI tab activation foundation.
- [Zen Mode](zen-mode.md) - GUI focused reading layout.

Each feature spec should include:

- Product behavior and user-facing requirements.
- Core, GUI, and TUI implementation notes.
- Important decisions and changed assumptions.
- Verification expectations and known gaps.

Update the relevant feature spec in the same change that modifies feature behavior.
