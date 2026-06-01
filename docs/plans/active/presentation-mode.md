# Presentation Mode Plan

## Goal

Add Phase 3 Presentation Mode while preserving PaperView's viewer-first default.
Users should explicitly enter a slide-focused view generated from Markdown
headings or rules, navigate slides, and keep the same document rendering
semantics used by the normal reader.

## Scope

- Shared core slide/deck model.
- TUI presentation navigation first, to validate the model quickly.
- GUI presentation view after the shared behavior is stable.
- Documentation updates for behavior, shortcuts, and verification.

Out of scope:

- Full `.tex` support via Tectonic.
- Knowledge Graph behavior.
- Presenter notes, speaker timer, fragment animations, or export-to-slides
  formats.
- A separate slide authoring language beyond Markdown headings/rules.

## Affected Paths

- `crates/paperview-core/src/`
- `crates/paperview-tui/src/`
- `crates/paperview-gui/src/`
- `docs/features/presentation-mode.md`
- `docs/TASKS.md`
- `docs/design/INDEX.md`
- `docs/arch/INDEX.md`

## Implementation Steps

1. Add core deck/slide generation from Markdown source.
2. Add focused core tests for slide boundaries and titles.
3. Add TUI Presentation Mode entry and slide navigation.
4. Add GUI Presentation Mode entry and slide navigation.
5. Update docs, tracker, and plan progress as each slice lands.

## Verification Plan

- Focused core tests for deck generation.
- Focused TUI/GUI tests as frontend slices land.
- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

## Progress

- 2026-06-01: Plan opened for Phase 3 Presentation Mode after Editing Mode
  closeout.
- 2026-06-01: Added `paperview-core::presentation` with `PresentationDeck`,
  `Slide`, explicit rule splitting, top-level-heading fallback, title
  derivation, and focused core tests.
