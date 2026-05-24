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

The current TUI theme maps the same visual intent into Ratatui styles:

- Dark shell/header and muted helper text.
- Cream active reader and active tab styles where terminal color support allows.
- Charcoal reader text with bold headings.
- Blue accents for quotes, Zen badges, active TOC entries, and selected lists.
- Distinct selected and matched search highlight styles.

Core does not own theme behavior.
The shared config stores `theme = "hybrid"` as the current supported theme preference so future theme switching can validate against a typed setting.

## Implementation Notes

- GUI color and container styles live in `crates/paperview-gui/src/theme.rs`.
- The Iced shell consumes theme styles from `app.rs` and `reader.rs`.
- TUI color styles live in `crates/paperview-tui/src/theme.rs`.
- The Ratatui app and render modules consume TUI theme helpers instead of
  scattering inline style literals.
- `paperview-core::ThemePreference` currently supports `Hybrid` and is serialized as `theme = "hybrid"` in the config file.
- The active document tab is visual scaffolding for future tabbed interface work; it currently represents a single open document.
- The initial theme follows the color values in `docs/design/INDEX.md`.

## Open Decisions

- Font embedding is deferred until typography work can choose durable bundled fonts.
- Additional theme variants and user-facing theme switching are deferred.

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
