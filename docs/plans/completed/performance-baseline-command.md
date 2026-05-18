# Performance Baseline Command

## Goal

Add a small repeatable command for measuring the document load, parse, and TUI
render pipeline before starting performance optimization work.

## Scope

- `crates/paperview-tui/src/main.rs`
- README, architecture notes, quality checks, task tracker, and feature docs

## Implementation Steps

1. Add `paperview-tui perf <file>`.
2. Validate supported file types.
3. Measure source read time.
4. Measure `Document` parse/model construction time.
5. Measure TUI render-line generation time.
6. Print document shape and timing values.
7. Add formatting and report-shape tests.
8. Update docs and trackers.

## Outcome

PaperView now has a headless performance baseline command:

```sh
cargo run -p paperview-tui -- perf docs/PRD.md
```

The command reports source size, source lines, parsed blocks, headings, rendered
TUI lines, and read/parse/render/total durations. It intentionally avoids
threshold enforcement until more measurements exist.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-tui perf
cargo run -p paperview-tui -- perf docs/PRD.md
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
