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

The first TUI slice exposes Presentation Mode with `p` from the reader. It
renders the active slide with PaperView's existing terminal Markdown renderer,
uses `Space`, `Right`, or `n` for the next slide, `Left` or `b` for the
previous slide, `Home` and `End` to jump to the first or last slide, and `Esc`
or `q` to return to the normal reader. The presentation pane title shows slide
progress, and the header shows the current slide title.

## Implementation Notes

- Core owns deck construction in `paperview-core::presentation`.
- `PresentationDeck` contains ordered `Slide` values.
- `Slide` stores a display title and the Markdown source for that slide.
- Frontends should stay thin: they own presentation navigation, layout, and
  shortcuts while core owns slide boundaries and titles.
- The first frontend slice proves the shared model in TUI before adding GUI
  presentation chrome.

## Open Decisions

- Whether slide breaks should eventually support presenter notes, fragments, or
  front matter.
- Whether GUI and TUI should share a dedicated presentation navigation state in
  core after the first frontend slice.
- Whether H2 headings should optionally split slides in a configurable mode.

## Verification Expectations

- Core tests cover explicit rule splitting, H1 fallback splitting, plain
  one-slide documents, empty separator chunks, and empty source.
- TUI tests cover entering Presentation Mode, rendering the first slide,
  next/previous navigation, bounds clamping, `Space` advance, first/last slide
  jumps, progress title text, and exit.
- GUI tests should cover presentation entry and navigation once GUI support
  lands.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, focused tests, and
  `cargo test --workspace` when frontend behavior changes.
