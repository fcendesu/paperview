# HTML Export Styling

## Goal

Make standalone HTML exports look like PaperView documents instead of plain
browser defaults.

## Completed

- Added semantic export classes for the paper surface, source panels, Mermaid
  panels, media blocks, data tables, callouts, and task lists.
- Expanded the embedded CSS with PaperView dark-shell and cream-reader tokens.
- Kept the export standalone with no external assets.
- Added core export assertions for the CSS theme and semantic class output.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core export`
- `cargo test -p paperview-tui export`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
