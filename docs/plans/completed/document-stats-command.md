# Document Stats Command

## Goal

Add a headless `stats <file>` command that prints document metadata without launching the TUI.

## Scope

- `crates/paperview-core/src/stats.rs`
- `crates/paperview-core/src/document.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-tui/src/main.rs`
- README, task tracker, and feature docs

## Implementation Steps

1. Added a shared document statistics model in core.
2. Counted words, lines, characters, headings, and estimated reading time.
3. Exposed stats through `Document::stats`.
4. Added `paperview-tui stats <file>` output without launching Ratatui.
5. Added focused core and CLI formatting tests.
6. Updated docs and trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core stats
cargo test -p paperview-tui stats
```

Full workspace checks were also run before completion.

## Outcome

The TUI binary now supports a headless document stats report. Richer output formats and AST-derived plain-text stats remain deferred.
