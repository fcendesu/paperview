# File Opening

## Product Behavior

PaperView accepts Markdown and plain-text documents from disk for direct reader
opening. The current core-supported reader extensions are:

- `.md`
- `.markdown`
- `.txt`

Unsupported extensions are rejected before disk reads. Supported files are read as UTF-8 text and wrapped in a `Document` with source text, optional path, parsed Markdown blocks, and a title derived from the first level-one heading when present.

Core also recognizes `.tex` as a supported compiled-document file type for the
Tectonic plan, but `.tex` files are not opened through `Document::open` or
parsed as Markdown. They must go through the explicit `.tex` compile path.

## Implementation Notes

- Core file loading lives in `paperview-core::Document::open`.
- `SupportedFileType` centralizes extension detection so GUI, TUI, drag-and-drop, and future CLI entrypoints can share the same allowlist.
- `SupportedFileType::Tex` is reserved for the Tectonic compile path and is not
  a Markdown reader document.
- File read errors preserve the source path and underlying `std::io::Error`.
- `paperview-tui [file]` loads one file into an interactive Ratatui terminal shell.
- `paperview-gui [file]` opens a native Iced window and renders the loaded document. Launching without a file shows an empty state.
- GUI drag-and-drop uses the same `Document::open` path as launch and history open flows.

## Open Decisions

- Binary/non-UTF-8 recovery behavior is deferred until richer launch flows exist.
- Multi-file drag-and-drop behavior is deferred until tabs exist.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
