# PaperView v0.1 Release Checklist

This checklist tracks the last-mile validation for a v0.1 release candidate.
It is the release-focused companion to `docs/quality/CHECKS.md`.

## Required Local Gates

Run before tagging or packaging a release candidate:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

## Smoke Commands

Run the user-facing command surface against a repository document:

```sh
cargo run -p paperview-tui -- stats docs/PRD.md
cargo run -p paperview-tui -- stats docs/PRD.md --json
cargo run -p paperview-tui -- search docs heading
cargo run -p paperview-tui -- export docs/PRD.md --to html
cargo run -p paperview-tui -- export docs/PRD.md --to pdf
cargo run -p paperview-tui -- perf docs/PRD.md
cargo run -p paperview-tui -- perf startup
cargo run -p paperview-tui -- perf startup docs/PRD.md
cargo run -p paperview-gui -- perf startup
cargo run -p paperview-gui -- perf startup docs/PRD.md
cargo run -p paperview-tui -- config path
```

Remove generated smoke-test export artifacts when they are not intended to be
committed:

```sh
rm -f docs/PRD.html docs/PRD.pdf
```

## Interactive Smoke Checks

- Launch `cargo run -p paperview-tui -- docs/PRD.md` and verify reader startup,
  scrolling, tabs, search, TOC navigation, task-list toggle behavior on a
  disposable file, split view, Zen Mode, and open-path prompt behavior.
- Launch `cargo run -p paperview-gui -- docs/PRD.md` and verify reader startup,
  local links, search, TOC navigation, tabs, split view, Zen Mode, drag and
  drop, local image previews, and remote-image placeholder behavior.

## Packaging Baseline

- macOS arm64 release artifacts were refreshed on 2026-05-26 in
  `docs/quality/DEPENDENCIES.md`.
- The release build should remain native Rust binaries with no Electron,
  WebView, Node, Python, browser-renderer, or external PDF-renderer runtime
  requirement.

## Open Platform Gaps

- Repeat dependency and release artifact checks on Linux.
- Repeat dependency and release artifact checks on Windows.
- Decide final distribution shape for GUI and TUI binaries.
- Measure real GUI native-window/event-loop startup timing.
- Measure real terminal initialization/event-loop startup timing.
- Measure real scroll/frame timing beyond the deterministic headless scroll
  workload estimate.

## v0.1 Ready When

- All required local gates pass.
- Smoke commands pass on a clean working tree.
- Generated smoke artifacts are removed or intentionally committed.
- At least macOS arm64 release artifacts are built and documented.
- Any unverified Linux/Windows packaging gaps are documented as release notes or
  resolved before cross-platform distribution is claimed.
- `docs/TASKS.md`, feature specs, and quality docs match the shipped behavior.
