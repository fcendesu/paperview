# PDF Export Layout Polish

## Goal

Improve the dependency-light PDF backend without replacing it with a heavier
renderer.

## Completed

- Added line-level layout metadata for indentation and post-line spacing.
- Added a larger document title treatment.
- Indented list items, blockquotes, code, math, and diagram source lines.
- Switched page splitting from fixed line counts to vertical-space pagination.
- Added focused tests for title sizing, indentation, preserved text, and
  multi-page output.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core export`
- `cargo test -p paperview-tui export`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
