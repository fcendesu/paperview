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
  `flowchart` edge lists also get a native flowchart preview.
- The TUI renders Mermaid source with the original fenced-code shape.

This is still a foundation slice. PaperView does not yet implement full Mermaid
layout, validate Mermaid syntax, or provide export-specific diagram assets.

## Implementation Notes

- `paperview-core::parser::Block::Diagram` stores the normalized language and
  source.
- `paperview-core::parser::elements::diagram` owns Mermaid fence detection and
  source normalization plus the simple flowchart preview parser.
- Non-Mermaid fenced code blocks continue to use `Block::CodeBlock`.
- `paperview-gui::reader` renders parsed flowchart edges as native node rows and
  keeps the original Mermaid source visible below the preview.

## Open Decisions

- Decide whether full Mermaid rendering should use a native layout engine, a
  bundled renderer, or export-time assets.
- Decide whether TUI diagrams should remain source-only or get an alternate text
  preview for simple flowcharts.
- Decide how rendered Mermaid assets should participate in future export.

## Verification

- Parser tests cover Mermaid fence detection and non-Mermaid code preservation.
- Core diagram tests cover simple flowchart preview parsing.
- TUI tests cover Mermaid source output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
