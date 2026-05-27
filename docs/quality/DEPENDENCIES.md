# Dependency And Packaging Audit

This document tracks PaperView's dependency surface for the release-readiness
goal called "Zero-Dependency Build" in the implementation tracker. In practice,
the goal means a native single-binary user experience with no runtime service,
browser engine, or package-manager dependency after installation. It does not
mean the Rust workspace has no build-time crates.

## Current Direct Dependencies

`paperview-core`:

- `pulldown-cmark` - Markdown parsing, including tables, task lists, math
  events, and fenced code blocks.
- `notify` - native file-watching for live reload.
- `serde` - config and history serialization.
- `toml` - config and history file encoding.

`paperview-gui`:

- `iced` with `image` - native desktop UI and local image previews.
- `reqwest` with `rustls-tls` and default features disabled - bounded remote
  image fetching without OpenSSL.
- `paperview-core` - shared parser, models, config, history, watcher, search,
  stats, and export logic.

`paperview-tui`:

- `ratatui` - terminal UI widgets and layout.
- `crossterm` - terminal input/output backend.
- `serde_json` - JSON output for `paperview-tui stats --json`.
- `paperview-core` - shared parser, models, config, history, watcher, search,
  stats, and export logic.

## Audit Command

Run this before release-oriented dependency or packaging changes:

```sh
cargo tree --workspace --depth 1
```

For a deeper audit, run:

```sh
cargo tree --workspace
```

## Packaging Check

Run this before release packaging changes:

```sh
cargo build --release --workspace
```

## v0.1 Distribution Shape

The v0.1 distribution shape is one compressed archive per verified platform.
Each archive contains:

- `paperview-gui`
- `paperview-tui`
- `README.md`
- `LICENSE.md`

The archive keeps GUI and TUI as separate native binaries in one package. This
keeps the release simple, preserves the zero-runtime packaging goal, and avoids
claiming installer behavior before it exists.

Create the archive with:

```sh
scripts/package-release.sh
```

The script writes `target/dist/paperview-v0.1.0-<target-triple>.tar.gz`.
Platform-native installers, macOS `.app`/DMG packaging, signing,
notarization, Homebrew, and Linux/Windows package-manager metadata are deferred
until after the first v0.1 archive release.

The current verified v0.1 artifact scope is macOS arm64 only. Linux and Windows
remain unverified until platform packaging checks are run on those targets.

Current local packaging baseline, recorded on 2026-05-25 from macOS arm64:

| Binary | Path | Format | Size |
| :--- | :--- | :--- | ---: |
| GUI | `target/release/paperview-gui` | Mach-O 64-bit executable arm64 | 17M |
| TUI | `target/release/paperview-tui` | Mach-O 64-bit executable arm64 | 2.0M |

Current local packaging baseline, refreshed on 2026-05-26 from macOS arm64
with `rustc 1.95.0` and `cargo 1.95.0`:

| Binary | Path | Format | Size | Bytes |
| :--- | :--- | :--- | ---: | ---: |
| GUI | `target/release/paperview-gui` | Mach-O 64-bit executable arm64 | 17M | 17,385,344 |
| TUI | `target/release/paperview-tui` | Mach-O 64-bit executable arm64 | 2.0M | 2,090,224 |

The 2026-05-26 release build completed with:

```sh
cargo tree --workspace --depth 1
cargo build --release --workspace
```

Current local v0.1 archive baseline, recorded on 2026-05-27 from macOS arm64:

| Artifact | Path | Format | Size |
| :--- | :--- | :--- | ---: |
| Archive | `target/dist/paperview-v0.1.0-aarch64-apple-darwin.tar.gz` | gzip-compressed tar archive | 7.4M |
| GUI | `target/dist/paperview-v0.1.0-aarch64-apple-darwin/paperview-gui` | Mach-O 64-bit executable arm64 | 17M |
| TUI | `target/dist/paperview-v0.1.0-aarch64-apple-darwin/paperview-tui` | Mach-O 64-bit executable arm64 | 2.0M |

The 2026-05-27 archive build completed with:

```sh
scripts/package-release.sh
tar -tzf target/dist/paperview-v0.1.0-aarch64-apple-darwin.tar.gz
file target/dist/paperview-v0.1.0-aarch64-apple-darwin/paperview-gui target/dist/paperview-v0.1.0-aarch64-apple-darwin/paperview-tui
```

## Current Assessment

- The project remains native Rust and avoids Electron, WebView, Node, Python,
  or external renderer runtime requirements.
- GUI remote image fetching uses `rustls-tls` with default `reqwest` features
  disabled to avoid platform OpenSSL runtime coupling.
- The PDF writer is dependency-light and does not require an external PDF or
  browser renderer.
- Mermaid and LaTeX support currently use parser/source-preserving previews
  instead of external rendering engines.

## Open Release Questions

- Repeat release binary size and platform packaging checks for Linux and
  Windows before v0.1.
- Revisit any future rich rendering dependency against native/offline packaging
  goals before adding it.
