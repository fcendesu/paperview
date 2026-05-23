# Performance Memory Estimate

## Goal

Make the headless perf command report a deterministic memory signal against the
MVP memory target.

## Completed

- Added source/model/rendered-text payload byte accounting to
  `paperview-tui perf`.
- Added a memory target line using the MVP 100MB target.
- Added byte-unit formatting for bytes, KiB, and MiB.
- Added focused tests for report formatting, byte formatting, and measured
  report shape.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui perf`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
