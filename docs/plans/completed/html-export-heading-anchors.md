# HTML Export Heading Anchors

## Goal

Make exported HTML headings linkable with stable anchors that match PaperView's
existing table-of-contents slug policy.

## Scope

- `crates/paperview-core/src/export.rs`
- `docs/features/html-export.md`
- `docs/features/table-of-contents.md`
- `docs/features/inline-span-rendering.md`
- `docs/arch/INDEX.md`
- `README.md`

## Outcome

- `paperview-core::export_html` now emits `id` attributes on exported heading
  tags.
- Heading ids reuse `ParsedDocument::toc` slugs, including duplicate-safe
  suffixes such as `details-2`.
- Export tests cover basic heading ids, escaped heading text, and duplicate
  heading anchors.

## Verification

- `cargo fmt --all`
- `cargo test -p paperview-core export`
- `cargo test -p paperview-tui export`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
