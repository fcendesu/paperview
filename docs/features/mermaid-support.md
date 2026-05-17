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
- The GUI renders Mermaid source in a diagram panel.
- The TUI renders Mermaid source with the original fenced-code shape.

This is a foundation slice. PaperView does not yet render Mermaid diagrams into
graphics, validate Mermaid syntax, or provide export-specific diagram assets.

## Implementation Notes

- `paperview-core::parser::Block::Diagram` stores the normalized language and
  source.
- `paperview-core::parser::elements::diagram` owns Mermaid fence detection and
  source normalization.
- Non-Mermaid fenced code blocks continue to use `Block::CodeBlock`.

## Open Decisions

- Choose the native diagram rendering path for the GUI.
- Decide whether TUI diagrams should remain source-only or get an alternate text
  preview for simple flowcharts.
- Decide how rendered Mermaid assets should participate in future export.

## Verification

- Parser tests cover Mermaid fence detection and non-Mermaid code preservation.
- TUI tests cover Mermaid source output.
- Workspace verification should include `cargo fmt --all`,
  `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
