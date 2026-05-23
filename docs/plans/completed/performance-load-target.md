# Performance Load Target

## Goal

Make the headless perf command report whether document read, parse, and render
time stays within the MVP load target.

## Completed

- Added a 500ms load target to `paperview-tui perf`.
- Reported load target status from measured total read/parse/render duration.
- Added report formatting and measured report-shape coverage.
- Clarified that this is a headless load target, not full GUI or interactive
  terminal startup timing.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-tui perf`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
