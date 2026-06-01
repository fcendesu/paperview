# Presentation Mode

## Current Behavior

Presentation Mode is complete for the current Phase 3 roadmap scope. It uses a
shared core slide model that can turn Markdown source into a presentation deck
without involving a frontend.

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

The GUI exposes Presentation Mode from the header with a `Present` button and
`Cmd/Ctrl+P`. It renders the active slide through the existing GUI reader
pipeline, hides the tab strip for a slide-focused layout, preserves the active
document path for slide resources, and uses header previous/next controls plus
a `View` button to return to the normal reader. Keyboard controls mirror the
TUI where practical: `Space`, `Right`, or `n` advance; `Left` or `b` move
backward; `Home` and `End` jump to the first or last slide; `Esc` exits
Presentation Mode.

## Implementation Notes

- Core owns deck construction in `paperview-core::presentation`.
- `PresentationDeck` contains ordered `Slide` values.
- `Slide` stores a display title and the Markdown source for that slide.
- Frontends should stay thin: they own presentation navigation, layout, and
  shortcuts while core owns slide boundaries and titles.
- TUI and GUI both reuse their normal Markdown renderers for slide content.

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
- GUI tests cover presentation entry, path preservation for slide resources,
  previous/next navigation bounds, first/last jumps, exit back to reader state,
  `Cmd/Ctrl+P` shortcut routing, and presentation keyboard navigation.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, focused tests, and
  `cargo test --workspace` when frontend behavior changes.
