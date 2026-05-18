# GUI Clickable Links Plan

## Goal

Make GUI inline links clickable anywhere rich inline text is rendered while
preserving the existing TUI Markdown-shaped display behavior.

## Scope

- Attach Iced rich-text link metadata to inline link spans.
- Emit a GUI message when a link span is clicked.
- Open absolute URLs and file paths through the platform default opener.
- Resolve relative links against the active document path when available.
- Document the GUI-only interaction behavior and remaining link gaps.

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- GUI launch smoke check with inline links

## Outcome

Completed clickable GUI inline links. The reader attaches Iced link metadata to
inline link spans across headings, paragraphs, lists, blockquotes, and table
cells. The GUI resolves relative targets from the active document path and opens
links through the platform default opener. In-document anchor navigation remains
deferred.
