# Workspace Search

## Product Behavior

PaperView can run a headless workspace search without launching the TUI:

```sh
cargo run -p paperview-tui -- search PaperView docs
```

It can also launch an interactive Ratatui result picker:

```sh
cargo run -p paperview-tui -- search PaperView docs --interactive
```

The command accepts:

- `search <query>` to search the current directory.
- `search <query> <path>` to search a specific file or directory.
- `--interactive` after the query or path to browse results in the TUI.

Results print as `path:line:column: text`. If there are no matches, the command prints `No matches`.
In interactive mode, results are listed with their source location and line preview. `j`/`k` or arrow keys move through matches, `Enter` opens the selected file near the matched source line, and `q`/`Esc` exits the result picker.

## Implementation Notes

- `paperview-core::search_workspace` invokes `rg` with parseable vimgrep output.
- `WorkspaceSearchMatch` stores path, line number, column, and matched line text.
- `paperview-tui search <query> [path]` formats results without initializing Ratatui.
- `paperview-tui search <query> [path] --interactive` initializes Ratatui with a search-result list and reuses the reader view when a result is opened.
- Empty queries return no matches.

## Decisions And Gaps

- Ripgrep must be installed and available as `rg`.
- Interactive result opening scrolls near the matched source line; exact rendered-line mapping is deferred.
- Result parsing uses ripgrep's vimgrep output shape.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core search
cargo test -p paperview-tui search
cargo run -p paperview-tui -- search PaperView docs
cargo run -p paperview-tui -- search PaperView docs --interactive
```

Run workspace checks before finishing workspace-search changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
