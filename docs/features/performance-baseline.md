# Performance Baseline

## Product Behavior

PaperView can print a compact headless performance baseline for a document:

```sh
cargo run -p paperview-tui -- perf docs/PRD.md
```

PaperView can also print a startup baseline for the TUI dashboard or for opening
a specific reader document:

```sh
cargo run -p paperview-tui -- perf startup
cargo run -p paperview-tui -- perf startup docs/PRD.md
```

The report includes:

- File path
- Source byte count
- Source line count
- Parsed block count
- Heading count
- Rendered TUI line count
- Scroll workload estimate with viewport count, synthetic scroll steps, average
  lines per viewport, and target status
- Estimated memory for source, parsed text payloads, and rendered TUI lines
- Memory target status for the MVP 100MB target
- Load target status for the MVP startup target
- Config and recent-history load timings
- Read, parse/model, render, and total durations

The startup report includes:

- Startup target, either `dashboard` or `reader`
- Reader file path when a document is provided
- Reader document count, rendered TUI line count, TOC item count, and watcher
  state
- Dashboard history entry count and selected history entry
- Document-open duration for reader startup
- App-state construction duration and total startup duration
- Startup target status against the MVP 500ms goal

## Implementation Notes

- The TUI binary owns the first `perf <file>` command.
- The TUI binary also owns `perf startup [file]`, which constructs the same
  dashboard or reader app state used by the interactive TUI without entering
  the alternate-screen terminal event loop.
- The command validates supported file types, loads config and recent-history
  stores, reads the source, constructs a `paperview_core::Document`, and renders
  TUI lines through `render_document_with_anchors`.
- The memory estimate is deterministic payload accounting from source text,
  parsed text strings, and rendered TUI lines. It is not process RSS.
- The scroll workload estimate is deterministic accounting from rendered TUI
  line count and a fixed 40-line viewport. It is not a real terminal frame-rate
  benchmark.
- Timings use `std::time::Instant` and are intended as local baseline signals,
  not deterministic benchmark assertions.
- Tests cover report formatting and report shape rather than exact timing
  values.
- Current local samples on 2026-05-25:
  - `perf startup`: dashboard app-state startup reported under 10ms with 3
    history entries.
  - `perf startup docs/PRD.md`: reader startup reported under 10ms total, 177
    rendered TUI lines, 23 TOC items, and an enabled file watcher.

## Decisions And Gaps

- This is a baseline command, not a full benchmark harness.
- GUI startup, GUI widget layout, terminal initialization/event-loop timing, OS
  RSS memory use, and real scroll frame timing remain unmeasured.
- The load target is reported from a startup-adjacent headless path that
  includes config load, history load, read, parse/model construction, and TUI
  render. `perf startup [file]` now measures interactive TUI app-state
  construction but still does not initialize the alternate-screen terminal or
  GUI.
- Historical baseline storage and threshold enforcement are deferred.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-tui perf
cargo test -p paperview-tui startup
cargo run -p paperview-tui -- perf docs/PRD.md
cargo run -p paperview-tui -- perf startup
cargo run -p paperview-tui -- perf startup docs/PRD.md
```

Run workspace checks before finishing performance command changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
