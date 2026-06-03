# Release Readiness Plan

## Goal

Refresh v0.1 release confidence after the completed Bookmarks and Tectonic
`.tex` slices, keeping platform claims honest and recording current local
quality, smoke, performance, dependency, and packaging evidence.

## Scope

- Re-run required local gates on macOS arm64.
- Re-run non-interactive CLI smoke commands against repository fixtures.
- Refresh dependency and release artifact baselines.
- Refresh headless startup, memory, and scroll workload notes.
- Record the current Tectonic doctor smoke result.
- Keep Linux/Windows packaging, real GUI/TUI event-loop timing, and real frame
  timing as open platform gaps unless they are actually verified.

## Affected Paths

- `docs/TASKS.md`
- `docs/quality/CHECKS.md`
- `docs/quality/RELEASE_CHECKLIST.md`
- `docs/quality/DEPENDENCIES.md`
- `docs/plans/active/release-readiness.md`

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo tree --workspace --depth 1
cargo build --release --workspace
scripts/package-release.sh
cargo run -p paperview-tui -- tex doctor
```

Run the documented non-interactive smoke commands from
`docs/quality/RELEASE_CHECKLIST.md`, then remove generated export artifacts.

## Progress

- 2026-06-03: Opened the release-readiness plan after completing the first
  Bookmarks and Tectonic `.tex` roadmap slices.
- 2026-06-03: Passed the documented non-interactive smoke commands on macOS
  arm64: stats, JSON stats, workspace search, HTML/PDF export, document perf,
  TUI startup perf, GUI startup perf, config path, and Tectonic doctor. Removed
  generated `docs/PRD.html` and `docs/PRD.pdf`.
- 2026-06-03: Refreshed dependency and packaging checks with
  `cargo tree --workspace --depth 1`, `cargo build --release --workspace`, and
  `scripts/package-release.sh`. The archive contained `paperview-gui`,
  `paperview-tui`, `README.md`, and `LICENSE.md`.
- 2026-06-03: Updated quality docs with current performance samples, release
  artifact sizes, and the latest `tex doctor` smoke result.
- 2026-06-03: Refreshed TUI interactive smoke on macOS arm64 with a PTY run:
  reader startup, scrolling, search, TOC jump, split view and resizing, Zen
  Mode, open-path prompt, isolated bookmark creation, clean quit, and
  disposable-file task toggle all passed.
- 2026-06-03: Ran a limited GUI smoke refresh. `paperview-gui docs/PRD.md`
  launched under isolated stores and the full `cargo test -p paperview-gui`
  suite passed. Display capture and scripted keystrokes were blocked by the
  local environment, so a visual/manual GUI interactive pass remains open.
