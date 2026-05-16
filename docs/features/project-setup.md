# Project Setup

## Product Behavior

PaperView is organized as a Rust workspace with shared document logic in `paperview-core` and separate native frontends for GUI and TUI use.

The current executable shells are intentionally minimal. They verify that each frontend can depend on the core crate before user-facing file opening or rendering behavior is added.

## Implementation Notes

- `crates/paperview-core` owns shared document and parser scaffolding.
- `crates/paperview-gui` is the future Iced desktop frontend.
- `crates/paperview-tui` is the future Ratatui terminal frontend.
- The root `Cargo.toml` is a virtual workspace manifest.
- Workspace lints forbid unsafe code and deny Clippy's default lint group.

## Open Decisions

- GUI dependencies should be introduced with the first real Iced app shell.
- TUI dependencies should be introduced with the first real Ratatui app shell.
- Markdown parsing dependencies should be introduced with the basic Markdown rendering slice.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
