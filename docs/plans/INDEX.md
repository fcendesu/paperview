# PaperView Execution Plans

This directory stores execution plans for complex implementation work.

Create a plan before work that spans multiple crates, changes architecture, introduces a major feature, or carries notable product/design risk.

## Layout

- `active/` - plans for work currently in progress.
- `completed/` - retained records of finished plans.
- `tech-debt-tracker.md` - known shortcuts, deferred cleanup, and follow-up work.

## Completed Plans

- [GUI Iced Shell](completed/gui-iced-shell.md) - first native GUI window with optional file loading and simple reader widgets.
- [Config Command](completed/config-command.md) - headless config path and edit commands.
- [History Sidebar Foundation](completed/history-sidebar-foundation.md) - shared recent-file model and first GUI history rail.
- [History Persistence](completed/history-persistence.md) - TOML-backed recent-file storage loaded and saved by the GUI.
- [HTML Export Command](completed/html-export-command.md) - headless HTML export from the parsed document model.
- [Image Rendering](completed/image-rendering.md) - standalone image metadata blocks and first GUI/TUI rendering.
- [In-Document Search Foundation](completed/in-document-search-foundation.md) - shared document search API and first TUI search workflow.
- [Inline Span Foundation](completed/inline-span-foundation.md) - paragraph bold, italic, code, and link span model.
- [Heading Inline Spans](completed/heading-inline-spans.md) - inline spans for Markdown headings.
- [Document Stats Command](completed/document-stats-command.md) - headless document metadata and heading-structure report.
- [List And Blockquote Inline Spans](completed/list-blockquote-inline-spans.md) - inline spans for list items and blockquotes.
- [Table Cell Inline Spans](completed/table-cell-inline-spans.md) - inline spans for Markdown table cells.
- [LaTeX Math Foundation](completed/latex-foundation.md) - source-preserving inline and display math support across core, GUI, and TUI.
- [LaTeX Readable Preview](completed/latex-readable-preview.md) - lightweight GUI display-math previews.
- [GUI History Open](completed/gui-history-open.md) - clickable GUI history entries that reopen and persist recent documents.
- [GUI Clickable Links](completed/gui-clickable-links.md) - clickable GUI inline links through the platform opener.
- [GUI Anchor Links](completed/gui-anchor-links.md) - clicked `#slug` links jump to matching headings.
- [GUI Image Previews](completed/gui-image-previews.md) - local bitmap previews for standalone images.
- [GUI Search Foundation](completed/gui-search-foundation.md) - header in-document search with match navigation.
- [GUI Search Highlighting](completed/gui-search-highlighting.md) - highlighted GUI search matches in rendered reader text.
- [GUI Search Selected Highlight](completed/gui-search-selected-highlight.md) - stronger rendered highlight for the selected GUI search match.
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
- [Mermaid Foundation](completed/mermaid-foundation.md) - source-preserving Mermaid diagram blocks across core, GUI, and TUI.
- [Mermaid Flowchart Preview](completed/mermaid-flowchart-preview.md) - native GUI preview for simple Mermaid flowcharts.
- [PDF Export Foundation](completed/pdf-export-foundation.md) - shared export format contract with PDF unavailable-backend handling.
- [Performance Baseline Command](completed/performance-baseline-command.md) - headless read, parse, and TUI render timing report.
- [TUI Live Reload](completed/tui-live-reload.md) - Ratatui active-document reload using the core watcher.
- [TUI TOC Scroll Sync](completed/tui-toc-scroll-sync.md) - Ratatui reader highlights the active TOC section while scrolling.
- [TUI TOC Jump Mode](completed/tui-toc-jump-mode.md) - Ratatui TOC focus mode and jump-to-heading navigation.
- [TUI Search Highlighting](completed/tui-search-highlighting.md) - highlighted TUI search result lines.
- [TUI Tab Close](completed/tui-tab-close.md) - close the active Ratatui tab with keyboard fallback behavior.
- [TUI Tabs Foundation](completed/tui-tabs-foundation.md) - multi-file TUI tabs with keyboard switching.
- [Ratatui Shell](completed/ratatui-shell.md) - first interactive terminal reader shell.
- [Table Rendering](completed/table-rendering.md) - structured Markdown table parsing and first GUI/TUI rendering.
- [Task List Rendering](completed/task-list-rendering.md) - read-only checked and unchecked Markdown task-list rendering.
- [TUI History Dashboard](completed/tui-history-dashboard.md) - no-file Ratatui recent-files dashboard with selection and open.
- [Workspace Search Command](completed/workspace-search-command.md) - headless ripgrep-backed workspace search report.

Each plan should include:

- Goal and scope.
- Affected source paths and docs.
- Implementation steps.
- Verification plan.
- Progress notes or final outcome.

When a plan is completed, keep it as a record unless it becomes misleading. Update or remove stale plans immediately.
