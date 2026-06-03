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

The GUI binary has a matching app-state startup baseline that stops before
opening the native window:

```sh
cargo run -p paperview-gui -- perf startup
cargo run -p paperview-gui -- perf startup docs/PRD.md
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

The GUI startup report includes:

- Startup target, either `dashboard` or `reader`
- Reader file path when a document is provided
- App status, open document count, history entry count, active TOC item count,
  and remote image placeholder count
- App-state construction duration and total startup duration
- Startup target status against the MVP 500ms goal

## Implementation Notes

- The TUI binary owns the first `perf <file>` command.
- The TUI binary also owns `perf startup [file]`, which constructs the same
  dashboard or reader app state used by the interactive TUI without entering
  the alternate-screen terminal event loop.
- The GUI binary owns `perf startup [file]`, which constructs the same
  `PaperView` state used by the Iced application without opening the native
  window or entering the Iced event loop.
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
  - `paperview-gui perf startup`: dashboard app-state startup reported under
    10ms with 2 history entries.
  - `paperview-gui perf startup docs/PRD.md`: reader app-state startup reported
    under 10ms total, 23 active TOC items, and 0 remote image placeholders.
- Current local samples on 2026-05-26:
  - `perf docs/PRD.md`: total reported 6.76ms, 6,120 bytes, 146 source lines,
    60 parsed blocks, 23 headings, 177 rendered TUI lines, 137 synthetic scroll
    steps, 17.0KiB estimated memory, and all load/scroll/memory targets passing.
  - `perf startup`: dashboard app-state startup reported 1.07ms total with 1
    history entry.
  - `perf startup docs/PRD.md`: reader startup reported 8.33ms total, 1.95ms
    document open, 6.26ms app state, 177 rendered TUI lines, 23 TOC items, and
    an enabled file watcher.
  - `paperview-gui perf startup`: dashboard app-state startup reported 1.62ms
    total with 1 history entry and 0 open documents.
  - `paperview-gui perf startup docs/PRD.md`: reader app-state startup reported
    4.65ms total, 23 active TOC items, and 0 remote image placeholders.
- Current local samples on 2026-06-03:
  - `perf docs/PRD.md`: total reported 9.96ms, 7,410 bytes, 165 source lines,
    60 parsed blocks, 23 headings, 190 rendered TUI lines, 150 synthetic scroll
    steps, 20.0KiB estimated memory, and all load/scroll/memory targets passing.
  - `perf startup`: dashboard app-state startup reported 1.58ms total with 0
    history entries.
  - `perf startup docs/PRD.md`: reader startup reported 19.12ms total, 9.67ms
    document open, 9.17ms app state, 190 rendered TUI lines, 23 TOC items, and
    an enabled file watcher.
  - `paperview-gui perf startup`: dashboard app-state startup reported
    234.06ms total with 1 history entry and 0 open documents.
  - `paperview-gui perf startup docs/PRD.md`: reader app-state startup reported
    232.79ms total, 23 active TOC items, and 0 remote image placeholders.

## Decisions And Gaps

- This is a baseline command, not a full benchmark harness.
- GUI native-window/event-loop timing, GUI widget layout timing, terminal
  initialization/event-loop timing, OS RSS memory use, and real scroll frame
  timing remain unmeasured.
- The load target is reported from a startup-adjacent headless path that
  includes config load, history load, read, parse/model construction, and TUI
  render. `perf startup [file]` now measures interactive TUI app-state
  construction, and the GUI startup command measures Iced app-state
  construction, but neither initializes the platform event loop.
- Historical baseline storage and threshold enforcement are deferred.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-tui perf
cargo test -p paperview-tui startup
cargo run -p paperview-tui -- perf docs/PRD.md
cargo run -p paperview-tui -- perf startup
cargo run -p paperview-tui -- perf startup docs/PRD.md
cargo test -p paperview-gui startup
cargo run -p paperview-gui -- perf startup
cargo run -p paperview-gui -- perf startup docs/PRD.md
```

Run workspace checks before finishing performance command changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
