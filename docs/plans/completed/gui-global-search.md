# GUI Global Search Plan

## Goal

Close the remaining Phase 2 Global Search GUI gap by exposing the existing
ripgrep-backed workspace search from the Iced app.

## Scope

- Reuse `paperview-core::search_workspace` and `WorkspaceSearchMatch`.
- Add GUI state for workspace search query, results, running status, and errors.
- Show workspace results in the left shell rail near history.
- Open clicked results as tabs and scroll near the matched source line.
- Update feature, design, tracker, and README docs.

Out of scope:

- Replacing ripgrep with an embedded index.
- Exact rendered-line positioning for matches.
- Cross-workspace saved search history.

## Affected Paths

- `crates/paperview-gui/src/app.rs`
- `docs/features/workspace-search.md`
- `docs/TASKS.md`
- `docs/design/INDEX.md`
- `docs/plans/INDEX.md`
- `README.md`

## Implementation Steps

1. Add GUI workspace search state and async search task routing.
2. Add a left-rail workspace search panel with query input and result rows.
3. Open selected results and scroll near the matched source line.
4. Add focused GUI tests for result state and open-near-line behavior.
5. Update docs and run verification.

## Verification Plan

- `cargo fmt --all`
- `cargo test -p paperview-gui workspace_search`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

## Progress

- 2026-06-01: Plan opened after Presentation Mode closeout.
- 2026-06-01: Added GUI workspace search state, left-rail query/results UI,
  async `search_workspace` task routing, result click-to-open behavior, focused
  GUI tests, and docs updates.
