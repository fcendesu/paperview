# PaperView Feature Specs

This directory stores one specification file per major feature. Feature specs are durable implementation records, not chat transcripts.

## Inventory

- [Project Setup](project-setup.md) - workspace layout, crate boundaries, and initial core/frontend shells.
- [File Opening](file-opening.md) - supported document formats and core file loading behavior.
- [Basic Markdown Rendering](basic-markdown-rendering.md) - initial shared Markdown parse model for frontend renderers.

Each feature spec should include:

- Product behavior and user-facing requirements.
- Core, GUI, and TUI implementation notes.
- Important decisions and changed assumptions.
- Verification expectations and known gaps.

Update the relevant feature spec in the same change that modifies feature behavior.
