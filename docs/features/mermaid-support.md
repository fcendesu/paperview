# Mermaid Support

## Current Behavior

PaperView recognizes fenced code blocks marked as Mermaid diagrams.

````text
```mermaid
graph TD
  A-->B
```
````

- Mermaid fences are represented as dedicated parsed document diagram blocks.
- The GUI renders Mermaid source in a diagram panel. Simple `graph` and
  `flowchart` edge lists, including common labeled arrow forms, comments,
  class suffixes, and common node shapes, also get a native flowchart preview.
- The TUI renders Mermaid source with the original fenced-code shape. Simple
  `graph` and `flowchart` edge lists, including common labeled arrow forms,
  comments, class suffixes, and common node shapes, also get a compact text
  flowchart preview.
- HTML export renders the same simple flowchart preview above the preserved
  Mermaid source panel.

This is still a foundation slice. PaperView does not yet implement full Mermaid
layout, validate Mermaid syntax, or provide export-specific diagram assets.

## Implementation Notes

- `paperview-core::parser::Block::Diagram` stores the normalized language and
  source.
- `paperview-core::parser::elements::diagram` owns Mermaid fence detection and
  source normalization plus the simple flowchart preview parser, including
  common labeled edge forms such as `A -- yes --> B`, `A -. maybe .-> B`, and
  `A ==>|fast| B`, comment trimming, class suffixes such as `A:::entry`, and
  common node shapes such as `A((Start))`, `A[/Input/]`, `A[(Store)]`, and
  `A{{Done}}`.
- Non-Mermaid fenced code blocks continue to use `Block::CodeBlock`.
- `paperview-gui::reader` renders parsed flowchart edges as native node rows and
  keeps the original Mermaid source visible below the preview.
- `paperview-tui::render` renders parsed flowchart edges as compact text rows
  and keeps the original Mermaid source visible below the preview.
- HTML export renders parsed flowchart edges as static node/edge rows and keeps
  the original Mermaid source visible below the preview.

## Open Decisions

- Decide whether full Mermaid rendering should use a native layout engine, a
  bundled renderer, or export-time assets.
- Decide how full rendered Mermaid assets should participate in future export.

## Verification

- Parser tests cover Mermaid fence detection and non-Mermaid code preservation.
- Core diagram tests cover simple and labeled flowchart preview parsing, common
  node shape cleanup, comment trimming, and class suffix cleanup.
- TUI tests cover Mermaid source output and simple flowchart preview output.
- Export tests cover simple Mermaid flowchart preview output and source-only
  fallback for unsupported Mermaid diagrams.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
