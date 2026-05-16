# PaperView — Code Modularity Mandates

To reduce cognitive load and prevent "God Files," the following architectural rules apply:

## 1. Feature Isolation
- **Docs:** Each major feature MUST have its own specification file in `docs/features/<feature_name>.md`.
- **Code:** Each Markdown element (Math, Mermaid, Tables) MUST be implemented in its own module.
    - Path: `paperview-core/src/parser/elements/`

## 2. Component-Based Rendering
- Avoid giant `match` statements in the main UI loop.
- Use the **Registry Pattern** or specialized **Component Traits** so that adding a feature (like "Mermaid") only requires adding a new file and registering it, rather than modifying the core rendering logic.

## 3. Directory Structure (Decentralized)
```text
docs/
├── PRD.md              # Vision & Roadmap
├── TASKS.md            # Implementation Tracker
├── features/           # ONE FILE PER FEATURE
│   ├── tabs.md
│   ├── split_view.md
│   └── latex.md
├── design/             # Theme & Layout specs
└── arch/               # Implementation detail specs
```
