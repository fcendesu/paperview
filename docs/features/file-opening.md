# File Opening

## Product Behavior

PaperView accepts Markdown and plain-text documents from disk. The current core-supported extensions are:

- `.md`
- `.markdown`
- `.txt`

Unsupported extensions are rejected before disk reads. Supported files are read as UTF-8 text and wrapped in a `Document` with source text, optional path, parsed Markdown blocks, and a title derived from the first level-one heading when present.

## Implementation Notes

- Core file loading lives in `paperview-core::Document::open`.
- `SupportedFileType` centralizes extension detection so GUI, TUI, drag-and-drop, and future CLI entrypoints can share the same allowlist.
- File read errors preserve the source path and underlying `std::io::Error`.
- GUI and TUI entrypoints are not wired to command-line file opening yet.

## Open Decisions

- Binary/non-UTF-8 recovery behavior is deferred until real launch flows exist.
- Recent-file persistence belongs with the later history feature.

## Verification Expectations

Run:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
