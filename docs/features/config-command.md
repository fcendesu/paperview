# Config Command

## Product Behavior

PaperView can locate and open its TOML config file through headless commands:

```sh
cargo run -p paperview-tui -- config path
cargo run -p paperview-tui -- config edit
```

`config path` prints the resolved config path. `config edit` creates a default config file if needed, then opens it with the platform default opener.

## Implementation Notes

- `paperview-core::config::ConfigStore` owns config path resolution, loading, saving, and file creation.
- `PAPERVIEW_CONFIG_PATH` overrides the default config location.
- The default config currently stores `schema_version = 1`.
- The TUI binary handles config commands without initializing Ratatui.

## Decisions And Gaps

- This is the first config file foundation; user-facing settings are still deferred.
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
