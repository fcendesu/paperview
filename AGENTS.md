# PaperView Agent Guide

This file is a map, not an encyclopedia. Keep it short enough to fit in agent context, and keep durable project knowledge in `docs/`.

The repository knowledge base is the system of record. Do not rely on chat history or memory when a project document exists.

## Start Here

Read these files before making project decisions:

- `docs/PRD.md` - product vision, requirements, and roadmap.
- `docs/arch/INDEX.md` - technical architecture.
- `docs/arch/MODULARITY.md` - modularity rules.
- `docs/design/INDEX.md` - visual and interaction design.
- `docs/TASKS.md` - implementation tracker.
- `docs/features/INDEX.md` - feature-spec inventory and implementation records.
- `docs/plans/INDEX.md` - execution-plan inventory.
- `docs/plans/tech-debt-tracker.md` - known shortcuts, deferred cleanup, and follow-up work.
- `docs/quality/CHECKS.md` - required verification commands and quality gates.

Use feature specs from `docs/features/` and execution plans from `docs/plans/` when they exist. If a complex change lacks an execution plan, create one before implementation.

## Context Rules

- Treat `docs/` as the repository knowledge base and system of record.
- The docs are local repository files. Agents may read them directly whenever project context is needed.
- Retrieve context from the documented paths instead of guessing project state.
- Start with lean context, then pull specific feature specs only when needed.
- For any task touching product behavior, architecture, design, implementation status, or feature scope, read the relevant doc path before deciding.
- Keep long-running design decisions, architecture notes, and product specs in `docs/`, not in chat history.
- Record implementation progress, completed work, and changed assumptions in the relevant docs as part of the same change.
- Garden stale docs immediately: update, move, or delete documentation that no longer reflects real behavior.
- Do not grow this file into a manual. Add or update focused docs under `docs/` instead.

## Source Layout

- `crates/paperview-core` - shared logic library: parsing, document models, history, config, file watching, and headless behavior.
- `crates/paperview-gui` - Iced-based desktop frontend.
- `crates/paperview-tui` - Ratatui-based terminal frontend.
- `docs/features/` - one specification file per major feature.
- `docs/plans/active/` - execution plans for work currently in progress.
- `docs/plans/completed/` - retained records of completed execution plans.
- `docs/arch/` - architecture rules and system structure.
- `docs/design/` - visual design, layout, and interaction specifications.
- `docs/quality/` - verification, quality gates, performance targets, and release-readiness notes.

## Record Keeping

- Feature work belongs in `docs/features/<feature_name>.md`.
- Active complex implementation plans belong in `docs/plans/active/<plan_name>.md`.
- Completed plans move to `docs/plans/completed/<plan_name>.md` when finished.
- Deferred cleanup and known shortcuts belong in `docs/plans/tech-debt-tracker.md`.
- Progress status belongs in `docs/TASKS.md`.
- Architecture changes belong in `docs/arch/INDEX.md` or a focused architecture doc.
- Design changes belong in `docs/design/INDEX.md` or a focused design doc.
- Verification rules belong in `docs/quality/CHECKS.md`.
- Every feature record should capture current behavior, implementation notes, open decisions, and verification expectations.

## Development Rules

- Keep business logic in `paperview-core`; keep GUI and TUI crates thin.
- Follow the "One Feature = One File" rule.
- Create or update `docs/features/<feature_name>.md` for major features.
- Keep Markdown element implementations isolated under `paperview-core/src/parser/elements/`.
- Avoid large central match statements in UI/rendering code; prefer registries or focused component traits.
- Preserve native, fast, quiet desktop behavior. PaperView is a viewer first.

## Required Checks

Run these before finishing code changes:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

Run focused tests as appropriate when the touched code has test coverage or clear behavior to verify.

## Documentation Updates

After any `feat`, `fix`, or `refactor`, update the docs immediately:

- Update `docs/TASKS.md` for progress tracking.
- Update relevant `docs/features/*.md` files when feature behavior changes.
- Update `docs/arch/INDEX.md` when structure or architecture changes.
- Update `docs/design/INDEX.md` or focused design docs when UI/interaction behavior changes.
- Update `docs/plans/tech-debt-tracker.md` when shortcuts or deferred cleanup are introduced.
- Update `docs/quality/CHECKS.md` when verification expectations change.
- Remove or correct stale documentation instead of leaving contradictions.

## Git

Use Conventional Commits with a body and footer. Commit messages must include this trailer exactly once:

```text
Co-authored-by: Codex <noreply@openai.com>
```
