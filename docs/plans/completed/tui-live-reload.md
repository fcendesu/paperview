# TUI Live Reload

## Goal and Scope

Bring live reload to the Ratatui reader by reusing the core watcher.

This plan covered:

- Attach `paperview-core::watch_file` to the active TUI document.
- Poll terminal input without blocking watcher processing.
- Reload the active document when the watched file changes.
- Preserve scroll within the new document bounds.
- Surface reload or watcher failures in the TUI status area.
- Update live-reload docs and tracker status.

Out of scope:

- Dashboard live reload.
- Debounced event coalescing.
- Multi-document watching.
- Exact section/heading scroll restoration.

## Affected Paths

- `crates/paperview-core/src/watcher.rs`
- `crates/paperview-tui/src/app.rs`
- `docs/features/live-reload.md`
- `docs/TASKS.md`

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Smoke test:

```sh
cargo run -p paperview-tui -- <temp-doc>
```

Then edit `<temp-doc>` externally and confirm the TUI refreshes while remaining interactive.

## Final Outcome

- The TUI reader keeps a core watcher alive for the active file.
- The TUI event loop polls keyboard input on a short tick so watcher events are processed.
- File changes reload the active document, title, body, and TOC while preserving bounded scroll.
- The core watcher now emits the originally watched path after matching against canonical paths, so `/tmp` and `/private/tmp` aliases do not cause frontends to ignore events.
