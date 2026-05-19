# GUI Remote Image Previews

## Goal

Render standalone remote Markdown images in the GUI while preserving the
existing metadata fallback behavior.

## Scope

- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/reader.rs`
- `crates/paperview-gui/Cargo.toml`
- `Cargo.lock`
- `docs/features/image-rendering.md`
- `docs/design/INDEX.md`
- `docs/TASKS.md`
- `README.md`

## Outcome

- GUI state tracks remote `http://` and `https://` image URLs visible in the
  active reader and Split View panes.
- A state-driven subscription fetches remote image bytes asynchronously and
  stores loaded or failed results in the current GUI session.
- Reader image panels show loading text, rendered remote previews, or a concise
  failure message while keeping image metadata visible.
- Remote image previews are capped at 10 MB per image.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-gui image`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
