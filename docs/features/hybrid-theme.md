# Hybrid Theme

## Product Behavior

PaperView uses a hybrid visual theme: a dark native shell surrounds a cream reader surface. This creates a quiet desktop frame while keeping document reading high contrast and paper-like.

The current GUI theme includes:

- Dark app shell and header.
- Dark tab bar with a cream active document tab.
- Cream reader surface with charcoal text.
- Muted grey metadata text.
- Blue accent for active chrome and code labels.
- Light code, quote, and paper borders.

TUI theme work has not started. Core does not own theme behavior.

## Implementation Notes

- GUI color and container styles live in `crates/paperview-gui/src/theme.rs`.
- The Iced shell consumes theme styles from `app.rs` and `reader.rs`.
- The active document tab is visual scaffolding for future tabbed interface work; it currently represents a single open document.
- The initial theme follows the color values in `docs/design/INDEX.md`.

## Open Decisions

- Font embedding is deferred until typography work can choose durable bundled fonts.
- TUI color mapping should be implemented when the Ratatui shell becomes stateful.
- Theme switching is out of scope for the first MVP theme.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For visual smoke testing:

```sh
cargo run -p paperview-gui -- docs/PRD.md
```
