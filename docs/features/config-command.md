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
- `theme = "hybrid"`, the current supported GUI/TUI visual theme.
- `zen_mode`, used by the GUI and TUI readers at startup and saved when toggled.
- `split_primary_width`, used by GUI and TUI Split View at startup and saved when resized.
- Optional `tex_compiler_path`, used by the headless `.tex` compile command
  when Tectonic is not available as `tectonic` on `PATH`.

## Implementation Notes

- `paperview-core::config::ConfigStore` owns config path resolution, loading, saving, and file creation.
- `PAPERVIEW_CONFIG_PATH` overrides the default config location.
- The default config stores `schema_version = 1`, `theme = "hybrid"`, `zen_mode = false`, and `split_primary_width = 50`.
- `tex_compiler_path` is omitted by default. When present, it is serialized as
  a string path and passed to `paperview-core::TexCompileInput`.
- The TUI binary handles config commands without initializing Ratatui.
- The GUI and TUI readers load config at startup, fall back to defaults if loading fails, and persist Zen Mode plus Split View width changes.
- `paperview-core::ThemePreference` validates the shared theme setting. Unknown theme strings are rejected as config decode errors.
- Missing config fields deserialize from defaults for compatibility with older config files.

## Decisions And Gaps

- User-facing theme switching and richer settings are still deferred.
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
