# Config Command

## Product Behavior

PaperView can locate and open its TOML config file through headless commands:

```sh
cargo run -p paperview-tui -- config path
cargo run -p paperview-tui -- config edit
```

`config path` prints the resolved config path. `config edit` creates a default config file if needed, then opens it with the platform default opener.

The config file currently stores:

- `schema_version = 1`
- `zen_mode`, used by the TUI reader at startup and saved when toggled.
- `split_primary_width`, used by the TUI Split View at startup and saved when resized.

## Implementation Notes

- `paperview-core::config::ConfigStore` owns config path resolution, loading, saving, and file creation.
- `PAPERVIEW_CONFIG_PATH` overrides the default config location.
- The default config stores `schema_version = 1`, `zen_mode = false`, and `split_primary_width = 50`.
- The TUI binary handles config commands without initializing Ratatui.
- The TUI reader loads config at startup, falls back to defaults if loading fails, and persists Zen Mode plus Split View width changes.
- Missing config fields deserialize from defaults for compatibility with older config files.

## Decisions And Gaps

- GUI preferences are still deferred.
- `config edit` delegates to the platform opener rather than embedding an editor.
- Config and history still use separate store types because their files have different schemas and override environment variables.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core config
cargo test -p paperview-tui config
```

Run workspace checks before finishing config changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
