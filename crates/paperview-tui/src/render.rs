use paperview_core::{
    Document,
    parser::{Block, HeadingLevel, TableAlignment, TocItem},
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
        Block::Diagram { language, source } => {
            output.push_str("```");
            output.push_str(language);
            output.push('\n');
            output.push_str(source);
            if !source.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```\n");
        }
        Block::Image { alt, url, title } => {
            output.push_str("![");
            output.push_str(alt);
            output.push_str("](");
            output.push_str(url);
            if !title.is_empty() {
                output.push_str(" \"");
                output.push_str(title);
                output.push('"');
            }
            output.push_str(")\n");
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
        Block::Table {
            alignments,
            header,
            rows,
        } => render_table(alignments, header, rows, output),
        Block::Rule => output.push_str("---\n"),
    }
}

fn render_heading(level: HeadingLevel, text: &str, output: &mut String) {
    output.push_str(&"#".repeat(usize::from(level.as_depth())));
    output.push(' ');
    output.push_str(text);
    output.push('\n');
}

fn render_table(
    alignments: &[TableAlignment],
    header: &[String],
    rows: &[Vec<String>],
    output: &mut String,
) {
    let widths = table_widths(header, rows);

    if !header.is_empty() {
        render_table_row(header, &widths, alignments, output);
        render_table_separator(&widths, alignments, output);
    }

    for row in rows {
        render_table_row(row, &widths, alignments, output);
    }
}

fn table_widths(header: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    let column_count = rows
        .iter()
        .map(Vec::len)
        .chain(std::iter::once(header.len()))
        .max()
        .unwrap_or(0);
    let mut widths = vec![3; column_count];

    for (index, cell) in header.iter().enumerate() {
        widths[index] = widths[index].max(cell.chars().count());
    }

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.chars().count());
        }
    }

    widths
}

fn render_table_row(
    cells: &[String],
    widths: &[usize],
    alignments: &[TableAlignment],
    output: &mut String,
) {
    output.push('|');
    for (index, width) in widths.iter().enumerate() {
        let cell = cells.get(index).map(String::as_str).unwrap_or("");
        output.push(' ');
        output.push_str(&aligned_cell(
            cell,
            *width,
            alignments
                .get(index)
                .copied()
                .unwrap_or(TableAlignment::None),
        ));
        output.push_str(" |");
    }
    output.push('\n');
}

fn render_table_separator(widths: &[usize], alignments: &[TableAlignment], output: &mut String) {
    output.push('|');
    for (index, width) in widths.iter().enumerate() {
        let alignment = alignments
            .get(index)
            .copied()
            .unwrap_or(TableAlignment::None);
        let rule = match alignment {
            TableAlignment::Left => format!(":{:-<width$}", "-", width = width - 1),
            TableAlignment::Center => format!(":{:-<width$}:", "-", width = width - 2),
            TableAlignment::Right => format!("{:-<width$}:", "-", width = width - 1),
            TableAlignment::None => "-".repeat(*width),
        };
        output.push(' ');
        output.push_str(&rule);
        output.push_str(" |");
    }
    output.push('\n');
}

fn aligned_cell(cell: &str, width: usize, alignment: TableAlignment) -> String {
    match alignment {
        TableAlignment::Right => format!("{cell:>width$}"),
        TableAlignment::Center => {
            let padding = width.saturating_sub(cell.chars().count());
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), cell, " ".repeat(right))
        }
        TableAlignment::None | TableAlignment::Left => format!("{cell:<width$}"),
    }
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
    fn renders_mermaid_diagram_blocks() {
        let document = Document::from_source("```mermaid\ngraph TD\n  A-->B\n```");

        assert!(render_document(&document).contains("```mermaid\ngraph TD\n  A-->B\n```\n"));
    }

    #[test]
    fn renders_markdown_tables() {
        let document = Document::from_source(
            "| Feature | Status |\n| :--- | ---: |\n| GUI | Done |\n| TUI | In progress |",
        );

        let rendered = render_document(&document);

        assert!(rendered.contains("| Feature |"));
        assert!(rendered.contains("Status |"));
        assert!(rendered.contains("| :------ |"));
        assert!(rendered.contains("----------: |"));
        assert!(rendered.contains("| GUI"));
        assert!(rendered.contains("Done |"));
        assert!(rendered.contains("| TUI"));
        assert!(rendered.contains("In progress |"));
    }

    #[test]
    fn renders_markdown_images() {
        let document = Document::from_source("![Architecture](docs/arch.png \"System diagram\")");

        assert!(
            render_document(&document)
                .contains("![Architecture](docs/arch.png \"System diagram\")\n")
        );
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
