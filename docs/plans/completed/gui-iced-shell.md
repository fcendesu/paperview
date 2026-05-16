# GUI Iced Shell

## Goal and Scope

Replace the print-only GUI entrypoint with a minimal native Iced window that can open an optional file argument and render the current shared Markdown block model.

This plan covers only the first usable shell:

- Launch `paperview-gui` as a native window.
- Load `paperview-gui <file>` through `paperview-core::Document::open`.
- Show a quiet empty state when no file is provided.
- Render headings, paragraphs, blockquotes, code blocks, lists, and horizontal rules with basic PaperView styling.

Out of scope:

- Tabs, sidebars, drag-and-drop, live reload, TOC, and split view.
- Rich inline styling and syntax highlighting.

## Affected Paths

- `crates/paperview-gui/`
- `docs/features/file-opening.md`
- `docs/features/basic-markdown-rendering.md`
- `docs/TASKS.md`
- `docs/plans/completed/gui-iced-shell.md`

## Implementation Steps

1. Add `iced` to `paperview-gui`.
2. Introduce small GUI modules for app state, theme constants, and reader rendering.
3. Wire CLI file arguments into initial app state.
4. Update feature docs and implementation tracker.
5. Run formatting, Clippy, tests, and a launch smoke check.

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p paperview-gui
cargo run -p paperview-gui -- <sample.md>
```

## Progress Notes

- Started after frontend print-preview support landed.
- Added a minimal native Iced shell with optional file loading and simple reader widgets.
- Verified formatting, Clippy, workspace tests, and a short native launch smoke with a sample Markdown file.
