# Workspace Search

## Product Behavior

PaperView can run a headless workspace search without launching the TUI:

```sh
cargo run -p paperview-tui -- search PaperView docs
```

The command accepts:

- `search <query>` to search the current directory.
- `search <query> <path>` to search a specific file or directory.

Results print as `path:line:column: text`. If there are no matches, the command prints `No matches`.

## Implementation Notes

- `paperview-core::search_workspace` invokes `rg` with parseable vimgrep output.
- `WorkspaceSearchMatch` stores path, line number, column, and matched line text.
- `paperview-tui search <query> [path]` formats results without initializing Ratatui.
- Empty queries return no matches.

## Decisions And Gaps

- Ripgrep must be installed and available as `rg`.
- The first command prints results only; interactive result selection in the TUI is deferred.
- Result parsing uses ripgrep's vimgrep output shape.

## Verification Expectations

Run focused checks with:

```sh
cargo test -p paperview-core search
cargo test -p paperview-tui search
cargo run -p paperview-tui -- search PaperView docs
```

Run workspace checks before finishing workspace-search changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
