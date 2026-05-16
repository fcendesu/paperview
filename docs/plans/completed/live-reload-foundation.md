# Live Reload Foundation

## Goal and Scope

Add the first live-reload slice so the GUI refreshes the active document when its source file changes on disk.

This plan covered:

- Add a `notify`-backed watcher module to `paperview-core`.
- Expose core watcher events without coupling them to Iced.
- Add a GUI subscription for the active document path.
- Reload the active document on file-change events.
- Preserve history/sidebar state and show reload failures in status.
- Update feature and tracker docs.

Out of scope:

- TUI live reload.
- Scroll-position preservation beyond keeping the current app layout.
- Debounced batching for editors that emit multiple write events.
- Watching multiple tabs or split panes.

## Affected Paths

- `crates/paperview-core/src/watcher.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-core/Cargo.toml`
- `crates/paperview-gui/src/app.rs`
- `crates/paperview-gui/src/main.rs`
- `docs/features/live-reload.md`
- `docs/features/INDEX.md`
- `docs/TASKS.md`
- `docs/arch/INDEX.md`

## Verification Plan

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Smoke test:

```sh
PAPERVIEW_HISTORY_PATH=<temp-history> cargo run -p paperview-gui -- <temp-doc>
```

Then edit `<temp-doc>` externally and confirm the GUI stays running and reloads via the subscription path.

## Final Outcome

- `paperview-core` owns the `notify` watcher and emits frontend-neutral `WatchEvent` values.
- GUI subscribes to the active document path and reloads it after relevant file-system events.
- Successful reloads refresh the active document, TOC, title, and recent-file entry.
- Reload failures are surfaced in the status line.
