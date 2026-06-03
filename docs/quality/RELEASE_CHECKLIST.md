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

2026-05-27 local gate status:

- Passed `cargo build --release --workspace` on macOS arm64 after the GUI local
  document link fix.
- Passed the final pre-tag audit on macOS arm64: `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, and
  `scripts/package-release.sh`.

2026-06-03 local gate status:

- Passed `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace`, `cargo tree --workspace --depth 1`,
  `cargo build --release --workspace`, and `scripts/package-release.sh` on
  macOS arm64 after closing the Bookmarks and Tectonic `.tex` roadmap slices.

## Smoke Commands

Run the user-facing command surface against a repository document:

```sh
cargo run -p paperview-tui -- stats docs/PRD.md
cargo run -p paperview-tui -- stats docs/PRD.md --json
cargo run -p paperview-tui -- search heading docs
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

2026-05-26 local smoke status:

- Passed the non-interactive smoke command set against `docs/PRD.md` on macOS
  arm64.
- Corrected the workspace-search smoke command to use the documented
  `search <query> [path]` argument order.
- Removed generated `docs/PRD.html` and `docs/PRD.pdf` after the smoke pass.

2026-06-03 local smoke status:

- Passed the non-interactive smoke command set against `docs/PRD.md` on macOS
  arm64, including stats, JSON stats, workspace search, HTML export, PDF
  export, document perf, TUI startup perf, GUI startup perf, config path, and
  Tectonic doctor.
- Removed generated `docs/PRD.html` and `docs/PRD.pdf` after the smoke pass.
- `tex doctor` found `Tectonic 0.16.9` through PATH/default discovery and
  successfully compiled `docs/fixtures/.paperview/tex/minimal.pdf`.

## Interactive Smoke Checks

- Launch `cargo run -p paperview-tui -- docs/PRD.md` and verify reader startup,
  scrolling, tabs, search, TOC navigation, task-list toggle behavior on a
  disposable file, split view, Zen Mode, and open-path prompt behavior.
- Launch `cargo run -p paperview-gui -- docs/PRD.md` and verify reader startup,
  local links, search, TOC navigation, tabs, split view, Zen Mode, drag and
  drop, local image previews, and remote-image placeholder behavior.

2026-05-26 local TUI interactive smoke status:

- Passed reader startup, visible reader/TOC rendering, scrolling, search submit
  with `n`/`N` navigation, Zen Mode toggle, split view with two documents,
  open-path prompt opening `docs/TASKS.md` as a second tab, and clean quit.
- Passed task-list toggle behavior on a disposable `/tmp` Markdown file and
  removed the file afterward.
- GUI interactive smoke remains open.

2026-06-03 local TUI interactive smoke status:

- Passed reader startup, visible reader/TOC rendering, scrolling, search submit
  with `n`/`N` navigation, TOC focus and jump, split view with two documents,
  split resizing, Zen Mode toggle, open-path prompt opening
  `docs/features/bookmarks.md` as a third tab, bookmark creation with an
  isolated `/tmp` bookmark store, and clean quit.
- Passed task-list toggle behavior on a disposable `/tmp` Markdown file by
  scrolling to the task source line, pressing Space, verifying the status line,
  and confirming the file changed to `- [x] Verify task toggle`.
- Removed the disposable task file and isolated bookmark store afterward.
- GUI interactive smoke remains open for the 2026-06-03 release-readiness pass.

2026-05-26 local GUI interactive smoke status:

- Passed GUI launch through a temporary macOS app wrapper, reader startup,
  visible history/sidebar/TOC/reader rendering, search highlighting with
  previous/next navigation, TOC click navigation, history item opening,
  tab display, split view with two open documents, Zen Mode toggle, local image
  metadata rendering from an absolute document path, and remote-image placeholder
  rendering.
- Cleaned up the temporary app wrapper, temporary image smoke files, and restored
  the original local PaperView history file after the smoke pass.
- Confirmed local link click navigation on 2026-05-27 with a temporary local
  Markdown pair: clicking the relative link opened the target document as a new
  active PaperView tab.
- Confirmed native GUI drag/drop on 2026-05-27 by dragging a temporary
  `dropped.md` from Finder into the PaperView window and verifying it opened as
  a new active tab.
- GUI interactive smoke is complete for the documented v0.1 macOS arm64 local
  checks.

2026-06-03 local GUI smoke status:

- Launched `paperview-gui docs/PRD.md` with isolated history and bookmark
  stores; macOS process inspection confirmed a live `paperview-gui` process.
- The current environment blocked display capture (`screencapture` could not
  create an image from the display) and blocked scripted GUI keystrokes through
  System Events, so this pass did not claim a visual/pixel-level GUI smoke.
- Passed the full `cargo test -p paperview-gui` suite covering GUI state and
  event behavior for reader startup, local links, search, TOC navigation, tabs,
  split view and resizing, Zen Mode, bookmarks, `.tex` status/actions,
  drag/drop events, local/remote image handling, task toggles, and workspace
  search.
- Cleaned up the launched GUI process and isolated `/tmp` smoke stores.
- A visual/manual GUI interactive smoke remains open for the 2026-06-03
  release-readiness pass.

## Packaging Baseline

- Draft v0.1 release notes live in `docs/RELEASE_NOTES.md`.
- macOS arm64 release artifacts were refreshed on 2026-05-26 in
  `docs/quality/DEPENDENCIES.md`.
- The release build should remain native Rust binaries with no Electron,
  WebView, Node, Python, browser-renderer, or external PDF-renderer runtime
  requirement.
- v0.1 ships as one `.tar.gz` archive per verified platform containing
  `paperview-gui`, `paperview-tui`, `README.md`, and `LICENSE.md`.
- Create the v0.1 archive with `scripts/package-release.sh`; the script writes
  `target/dist/paperview-v0.1.0-<target-triple>.tar.gz`.
- Passed the v0.1 archive build on 2026-05-27 for macOS arm64. The archive
  contained the GUI binary, TUI binary, README, and license.
- Refreshed the v0.1 archive build on 2026-06-03 for macOS arm64. The archive
  contained the GUI binary, TUI binary, README, and license.

## Open Platform Gaps

- Repeat dependency and release artifact checks on Linux.
- Repeat dependency and release artifact checks on Windows.
- Do not claim Linux or Windows v0.1 release support until those packaging
  checks pass; the current verified artifact scope is macOS arm64 only.
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
