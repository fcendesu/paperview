# Performance Baseline

## Product Behavior

PaperView can print a compact headless performance baseline for a document:

```sh
cargo run -p paperview-tui -- perf docs/PRD.md
```

The report includes:

- File path
- Source byte count
- Source line count
- Parsed block count
- Heading count
- Rendered TUI line count
- Estimated memory for source, parsed text payloads, and rendered TUI lines
- Memory target status for the MVP 100MB target
- Read, parse/model, render, and total durations

## Implementation Notes

- The TUI binary owns the first `perf <file>` command.
- The command validates supported file types, reads the source, constructs a
  `paperview_core::Document`, and renders TUI lines through
  `render_document_with_anchors`.
- The memory estimate is deterministic payload accounting from source text,
  parsed text strings, and rendered TUI lines. It is not process RSS.
- Timings use `std::time::Instant` and are intended as local baseline signals,
  not deterministic benchmark assertions.
- Tests cover report formatting and report shape rather than exact timing
  values.

## Decisions And Gaps

- This is a baseline command, not a full benchmark harness.
- GUI startup, GUI widget layout, OS RSS memory use, and scroll frame timing
  remain unmeasured.
- Historical baseline storage and threshold enforcement are deferred.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-tui perf
cargo run -p paperview-tui -- perf docs/PRD.md
```

Run workspace checks before finishing performance command changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
