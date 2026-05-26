# PaperView Quality Checks

This document is the verification source of truth for implementation work.

For release-candidate validation, use `docs/quality/RELEASE_CHECKLIST.md` in
addition to the implementation checks below.

## Required For Code Changes

Run these before finishing any code change:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

## Tests

Run focused tests when touched code has test coverage or obvious behavior to verify.

Use broader workspace tests when a change affects shared core behavior, parser behavior, CLI behavior, or cross-crate contracts.

## Dependency Audit

For release-readiness or dependency-surface changes, check the direct workspace
dependency surface:

```sh
cargo tree --workspace --depth 1
```

Record durable dependency and packaging notes in
`docs/quality/DEPENDENCIES.md`.

## Packaging Readiness

Before release packaging changes, prove that optimized native artifacts build:

```sh
cargo build --release --workspace
```

Then record the produced GUI and TUI binary paths, platform format, and sizes in
`docs/quality/DEPENDENCIES.md`.

## Documentation

After any `feat`, `fix`, or `refactor`, update:

- `docs/TASKS.md` for implementation status.
- Relevant `docs/features/*.md` files for behavior or implementation changes.
- `docs/arch/INDEX.md` or focused architecture docs for structure changes.
- `docs/design/INDEX.md` or focused design docs for UI/interaction changes.
- `docs/plans/tech-debt-tracker.md` for intentional shortcuts or deferred cleanup.

## Performance Expectations

PaperView should preserve the product goals from `docs/PRD.md`:

- Cold startup under 500ms.
- Smooth 60 FPS scrolling for typical technical documents.
- Lower memory usage than Electron-style alternatives.

When performance-sensitive code changes, record the verification approach in the relevant feature spec or execution plan.

Use the headless perf command for a quick local document pipeline baseline:

```sh
cargo run -p paperview-tui -- perf docs/PRD.md
```

This measures config load, recent-history load, source read, document
parse/model construction, TUI line rendering, deterministic memory payloads, and
a headless scroll workload estimate.

Use the startup perf commands for dashboard and reader app-state baselines:

```sh
cargo run -p paperview-tui -- perf startup
cargo run -p paperview-tui -- perf startup docs/PRD.md
cargo run -p paperview-gui -- perf startup
cargo run -p paperview-gui -- perf startup docs/PRD.md
```

These commands do not yet measure platform event-loop/window timing, GUI widget
layout timing, OS memory, or real scroll frame timing.
