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
- 2026-05-27: Added the first GUI Editing Mode slice with header Edit/View and
  Save actions, a source editor plus live preview layout, and focused GUI tests
  for toggling and saving edits.
- 2026-05-30: Added GUI keyboard shortcuts: `Cmd/Ctrl+E` toggles Editing Mode
  and `Cmd/Ctrl+S` saves while editing.
- 2026-05-30: Added lightweight GUI editor Markdown syntax styling for common
  block markers, inline code, links, and emphasis without adding a new
  dependency.
- 2026-05-31: Added the first TUI Editing Mode slice with `e` to enter a
  terminal source buffer, append/backspace editing, `Ctrl+S` save, `Esc` return
  to reader, and focused TUI key-flow tests.
- 2026-05-31: Upgraded the TUI editor buffer with UTF-8-aware cursor movement,
  insertion at cursor, `Delete`, `Home` / `End`, vertical arrow movement, and
  focused cursor-editing tests.
- 2026-05-31: Added TUI live preview while editing by rendering
  `EditSession::preview_document` beside the source buffer and covering preview
  refresh before save in focused tests.
- 2026-05-31: Added TUI editor viewport state with cursor-visible scrolling,
  `PageUp` / `PageDown`, and focused tests for long edit buffers.
- 2026-05-31: Added TUI dirty-edit discard protection for `Esc`, tab switching,
  and tab close, with focused tests for warning, confirmation, and save reset.
- 2026-05-31: Added `Ctrl+P` TUI edit preview toggling for narrow terminals,
  with header status and focused visibility tests.
