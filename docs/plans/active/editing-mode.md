# Editing Mode Plan

## Goal

Add Phase 2 Editing Mode while preserving PaperView's viewer-first default.
Users should explicitly enter an editor surface, edit Markdown source, preview
the edited document, and save back to the source file.

## Scope

- Shared core edit-session state.
- GUI viewer/editor toggle with live preview and save.
- TUI edit affordance after the core and GUI behavior are stable.
- Documentation updates for behavior, shortcuts, and verification.

Out of scope:

- Full `.tex` editing or Tectonic integration.
- Knowledge Graph behavior.
- A general IDE-like editor or plugin system.
- Rich syntax-highlighting engine integration unless the lightweight first
  editor slice proves too weak.

## Affected Paths

- `crates/paperview-core/src/`
- `crates/paperview-gui/src/`
- `crates/paperview-tui/src/`
- `docs/features/editing-mode.md`
- `docs/TASKS.md`
- `docs/design/INDEX.md`
- `docs/arch/INDEX.md`

## Implementation Steps

1. Add core edit-session state with source buffer, dirty tracking, preview
   document generation, and file-backed save.
2. Add GUI Editing Mode toggle for the active document.
3. Add GUI editor/preview split and save action.
4. Add lightweight Markdown syntax styling if it can be done without a heavy
   dependency or custom editor rewrite.
5. Add TUI edit flow after the GUI path establishes the shared behavior.
6. Update docs, release notes, and tracker as each slice lands.

## Verification Plan

- Focused core tests for edit-session behavior.
- Focused GUI/TUI tests as frontend slices land.
- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

## Progress

- 2026-05-27: Plan opened. PRD and tracker scope updated to target Editing
  Mode and Presentation Mode while deferring Tectonic `.tex` and Knowledge
  Graph work.
- 2026-05-27: Added `paperview-core::EditSession` foundation with dirty-state
  tracking, live preview document generation, and file-backed save behavior.
