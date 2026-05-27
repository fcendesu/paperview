# Editing Mode

## Current Behavior

Editing Mode is the active Phase 2 focus. The first implementation slice adds
shared core edit-session state so GUI and TUI frontends can add editing
controls without each inventing save, dirty-state, or preview-refresh rules.

The intended user-facing behavior is:

- Toggle a file-backed document between read-only viewer mode and editor mode.
- Edit the Markdown source in an editor surface.
- See a live preview generated from the edited source.
- Save changes back to the source file.
- Preserve PaperView's viewer-first defaults; editing is an explicit mode, not
  the startup state.

## Implementation Notes

- Editing state belongs in `paperview-core` first.
- `paperview-core::EditSession` owns the optional file path, original source,
  editable buffer, dirty-state comparison, preview document generation, and
  file-backed save.
- `EditSession::preview_document` reparses the edited buffer into a normal
  `Document` so existing renderers can show live previews without a parallel
  document model.
- `EditSession::save` writes the buffer back to the original path and rejects
  pathless buffers.
- GUI and TUI frontends should stay thin: they own text input widgets,
  keyboard shortcuts, and layout, while core owns dirty-state and save
  semantics.
- The first GUI slice should prefer a focused editor/preview split for the
  active document over adding a general-purpose editor abstraction.
- The first TUI slice can be conservative: expose edit state and save behavior
  before attempting a rich terminal editor.

## Open Decisions

- Exact keyboard shortcut for toggling Editing Mode.
- Whether the first TUI editor is a line-oriented buffer or an external-editor
  handoff.
- How much Markdown syntax highlighting is needed for the first GUI editor
  slice.
- Whether save should preserve scroll position exactly or reset to the edited
  preview's nearest heading.

## Verification Expectations

- Core tests cover edit-session creation, dirty-state tracking, live preview
  document generation, file-backed save behavior, and missing-path save
  rejection.
- GUI tests should cover toggling into editor mode, editing text, preview
  refresh, and save status once the GUI slice lands.
- TUI tests should cover any edit-mode key flow once the TUI slice lands.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, focused tests, and
  `cargo test --workspace` when frontend behavior changes.
