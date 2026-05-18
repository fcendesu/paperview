# Workspace Search Command

## Goal

Add a headless workspace search command backed by ripgrep-style results.

## Scope

- `crates/paperview-core/src/search.rs`
- `crates/paperview-tui/src/main.rs`
- README, task tracker, and feature docs

## Implementation Steps

1. Added a shared workspace search result model in core.
2. Invoked `rg` with a stable parseable output format.
3. Parsed ripgrep result lines in focused unit tests.
4. Added `paperview-tui search <query> [path]` output without launching Ratatui.
5. Added docs and tracker updates.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core search
cargo test -p paperview-tui search
cargo run -p paperview-tui -- search PaperView docs
```

Full workspace checks were also run before completion.

## Outcome

The TUI binary now supports a headless ripgrep-backed workspace search report. Interactive result selection remains deferred.
