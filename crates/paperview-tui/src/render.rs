use paperview_core::{
    Document,
    parser::{Block, HeadingLevel, TocItem},
};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedDocument {
    pub lines: Vec<String>,
    pub block_line_starts: Vec<BlockLineStart>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockLineStart {
    pub block_index: usize,
    pub line: usize,
}

#[cfg(test)]
fn render_document(document: &Document) -> String {
    let mut output = render_document_lines(document).join("\n");
    output.push('\n');
    output.push_str(&render_toc_lines(&document.parsed().toc()).join("\n"));
    output.push('\n');

    output
}

#[cfg(test)]
pub fn render_document_lines(document: &Document) -> Vec<String> {
    render_document_with_anchors(document).lines
}

pub fn render_document_with_anchors(document: &Document) -> RenderedDocument {
    let mut output = String::new();
    let mut block_line_starts = Vec::new();

    for (block_index, block) in document.parsed().blocks.iter().enumerate() {
        block_line_starts.push(BlockLineStart {
            block_index,
            line: output.lines().count(),
        });
        render_block(block, &mut output);
        output.push('\n');
    }

    RenderedDocument {
        lines: output.lines().map(ToOwned::to_owned).collect(),
        block_line_starts,
    }
}

fn render_block(block: &Block, output: &mut String) {
    match block {
        Block::Heading { level, text } => render_heading(*level, text, output),
        Block::Paragraph(text) => {
            output.push_str(text);
            output.push('\n');
        }
        Block::BlockQuote(text) => {
            output.push_str("> ");
            output.push_str(text);
            output.push('\n');
        }
        Block::CodeBlock { language, code } => {
            output.push_str("```");
            if let Some(language) = language {
                output.push_str(language);
            }
            output.push('\n');
            output.push_str(code);
            if !code.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```\n");
        }
        Block::List { ordered, items } => {
            for (index, item) in items.iter().enumerate() {
                if *ordered {
                    output.push_str(&format!("{}. {item}\n", index + 1));
                } else {
                    output.push_str(&format!("- {item}\n"));
                }
            }
        }
        Block::Math { source, .. } => {
            output.push_str("$$\n");
            output.push_str(source);
            if !source.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("$$\n");
        }
        Block::Rule => output.push_str("---\n"),
    }
}

fn render_heading(level: HeadingLevel, text: &str, output: &mut String) {
    output.push_str(&"#".repeat(usize::from(level.as_depth())));
    output.push(' ');
    output.push_str(text);
    output.push('\n');
}

#[cfg(test)]
pub fn render_toc_lines(toc: &[TocItem]) -> Vec<String> {
    let mut lines = vec!["On this page".to_owned(), "------------".to_owned()];

    if toc.is_empty() {
        lines.push("No headings".to_owned());
        return lines;
    }

    for item in toc {
        let indent = "  ".repeat(usize::from(item.level.as_depth().saturating_sub(1)));
        lines.push(format!("{indent}- {}", item.title));
    }

    lines
}

pub fn render_toc_text(
    toc: &[TocItem],
    active_block_index: Option<usize>,
    selected_index: Option<usize>,
    is_focused: bool,
) -> Text<'static> {
    if toc.is_empty() {
        return Text::from(vec![Line::from(Span::styled(
            "No headings",
            Style::default().fg(Color::DarkGray),
        ))]);
    }

    let mut lines = Vec::new();

    for (index, item) in toc.iter().enumerate() {
        let is_active = active_block_index == Some(item.block_index);
        let is_selected = selected_index == Some(index);
        let indent = "  ".repeat(usize::from(item.level.as_depth().saturating_sub(1)));
        let marker = match (is_focused && is_selected, is_active) {
            (true, true) => "*",
            (true, false) => "+",
            (false, true) => ">",
            (false, false) => "-",
        };
        let style = if is_focused && is_selected {
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(Span::styled(
            format!("{indent}{marker} {}", item.title),
            style,
        )));
    }

    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use paperview_core::Document;

    use super::{render_document, render_document_with_anchors, render_toc_text};

    #[test]
    fn renders_basic_markdown_blocks() {
        let document = Document::from_source("# PaperView\n\n- Fast\n- Native\n\n---");

        assert_eq!(
            render_document(&document),
            "# PaperView\n\n- Fast\n- Native\n\n---\n\nOn this page\n------------\n- PaperView\n"
        );
    }

    #[test]
    fn renders_nested_toc_entries() {
        let document = Document::from_source("# PaperView\n\n## Reader\n\n### Navigation");

        assert!(
            render_document(&document).contains(
                "On this page\n------------\n- PaperView\n  - Reader\n    - Navigation\n"
            )
        );
    }

    #[test]
    fn renders_latex_math_blocks() {
        let document = Document::from_source("Before $x$.\n\n$$\nE = mc^2\n$$");

        assert!(render_document(&document).contains("Before $x$.\n\n$$\nE = mc^2\n$$\n"));
    }

    #[test]
    fn records_block_line_starts() {
        let document = Document::from_source("# PaperView\n\nBody.\n\n## Reader");
        let rendered = render_document_with_anchors(&document);

        assert_eq!(rendered.block_line_starts[0].block_index, 0);
        assert_eq!(rendered.block_line_starts[0].line, 0);
        assert_eq!(rendered.block_line_starts[2].block_index, 2);
        assert!(rendered.block_line_starts[2].line > rendered.block_line_starts[0].line);
    }

    #[test]
    fn highlights_active_toc_item() {
        let document = Document::from_source("# PaperView\n\n## Reader");
        let text = render_toc_text(&document.parsed().toc(), Some(1), None, false);
        let rendered = format!("{text:?}");

        assert!(rendered.contains("> Reader"));
    }

    #[test]
    fn marks_selected_toc_item_when_focused() {
        let document = Document::from_source("# PaperView\n\n## Reader");
        let text = render_toc_text(&document.parsed().toc(), Some(0), Some(1), true);
        let rendered = format!("{text:?}");

        assert!(rendered.contains("+ Reader"));
    }
}
