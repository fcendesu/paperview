# Config Command

## Goal

Add headless config commands for locating and opening PaperView's TOML config file.

## Scope

- `crates/paperview-core/src/config.rs`
- `crates/paperview-core/src/lib.rs`
- `crates/paperview-tui/src/main.rs`
- README, task tracker, and feature docs

## Implementation Steps

1. Added a small shared `Config` model and `ConfigStore`.
2. Resolved the default config path with an environment override.
3. Ensured the config file exists before editing.
4. Added `paperview-tui config path` and `paperview-tui config edit`.
5. Added focused core and CLI tests.
6. Updated docs and trackers.

## Verification

```sh
cargo fmt --all
cargo test -p paperview-core config
cargo test -p paperview-tui config
cargo run -p paperview-tui -- config path
```

Full workspace checks were also run before completion.

## Outcome

The TUI binary now supports headless config path and edit commands. User-facing settings are still deferred.
