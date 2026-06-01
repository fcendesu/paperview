# Presentation Mode

## Current Behavior

Presentation Mode is the active Phase 3 focus. The first implementation slice
adds a shared core slide model that can turn Markdown source into a
presentation deck without involving a frontend.

The current deck generation rules are:

- Prefer explicit slide breaks using Markdown thematic-rule lines: `---`,
  `***`, or `___`.
- If no explicit slide breaks are present, split on top-level `#` headings.
- Preserve each slide's Markdown source so GUI and TUI renderers can reuse the
  normal PaperView document pipeline.
- Derive slide titles from the first Markdown heading in the slide, falling
  back to the first non-empty line.
- Ignore empty slide chunks caused by leading, repeated, or trailing
  separators.

The intended user-facing milestone is a viewer-first presentation surface:
open a Markdown document, enter Presentation Mode explicitly, move between
slides, and render each slide with PaperView's existing Markdown support.

## Implementation Notes

- Core owns deck construction in `paperview-core::presentation`.
- `PresentationDeck` contains ordered `Slide` values.
- `Slide` stores a display title and the Markdown source for that slide.
- Frontends should stay thin: they own presentation navigation, layout, and
  shortcuts while core owns slide boundaries and titles.
- The first frontend slice should prove the shared model in TUI before adding
  GUI presentation chrome.

## Open Decisions

- Whether slide breaks should eventually support presenter notes, fragments, or
  front matter.
- Whether GUI and TUI should share a dedicated presentation navigation state in
  core after the first frontend slice.
- Whether H2 headings should optionally split slides in a configurable mode.

## Verification Expectations

- Core tests cover explicit rule splitting, H1 fallback splitting, plain
  one-slide documents, empty separator chunks, and empty source.
- TUI tests should cover entering Presentation Mode and moving between slides
  once the first TUI slice lands.
- GUI tests should cover presentation entry and navigation once GUI support
  lands.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, focused tests, and
  `cargo test --workspace` when frontend behavior changes.
