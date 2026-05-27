# PaperView Release Notes

## v0.1.0

Release status: prepared, not tagged or published.

Verified artifact scope:

- macOS arm64 archive built locally with `scripts/package-release.sh`.
- Archive path:
  `target/dist/paperview-v0.1.0-aarch64-apple-darwin.tar.gz`.
- Archive contents: `paperview-gui`, `paperview-tui`, `README.md`, and
  `LICENSE.md`.

Platform scope:

- macOS arm64 is the only locally verified v0.1 artifact target.
- Linux and Windows builds are not yet verified and should not be claimed as
  supported release artifacts until their packaging checks pass.

### Included

- Native Iced GUI reader with history, tabs, split view, TOC navigation, search,
  Zen Mode, local document links, task checkbox writeback, live reload, native
  drag/drop, local image previews, and remote-image placeholders.
- Ratatui TUI reader with recent-files dashboard, tabs, split view, TOC
  navigation, search, Zen Mode, task checkbox writeback, live reload, and
  open-path workflow.
- Shared Markdown parser support for headings, paragraphs, lists, blockquotes,
  code blocks, tables, task lists, inline bold/italic/code/link spans, images,
  LaTeX math previews, and Mermaid flowchart previews.
- Headless toolkit commands for document stats, workspace search, HTML/PDF
  export, config management, and performance baselines.
- Dependency-light HTML/PDF export without external browser, WebView, Node,
  Python, or PDF-renderer runtime requirements.

### Known Limitations

- macOS `.app`, DMG, signing, notarization, Homebrew, and installer packaging
  are deferred.
- Linux and Windows archives are deferred until platform packaging checks pass.
- Real native window/event-loop startup timing and real scroll frame timing are
  not yet measured; current performance notes rely on headless app-state and
  deterministic workload baselines.
- LaTeX and Mermaid support are readable/source-preserving foundations, not full
  rich renderers.
- PDF export is text-first and does not yet include rich diagram or formula
  rendering.
- Wide-table scrolling, exact GUI layout geometry, background-tab watching, and
  exact scroll restoration are deferred.

### Verification Summary

- Required Rust gates passed locally on macOS arm64:
  `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace`, and `cargo build --release --workspace`.
- Non-interactive smoke commands passed against `docs/PRD.md`.
- TUI interactive smoke passed for reader startup, scrolling, search, TOC
  navigation, task toggling on a disposable file, split view, Zen Mode,
  open-path behavior, and clean quit.
- GUI interactive smoke passed for reader startup, search, TOC navigation,
  history opening, tabs, split view, Zen Mode, local image metadata, remote
  image placeholders, local document link opening, and native drag/drop.
