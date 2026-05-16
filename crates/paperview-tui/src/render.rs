use paperview_core::{
    Document,
    parser::{Block, HeadingLevel, TocItem},
};

#[cfg(test)]
fn render_document(document: &Document) -> String {
    let mut output = render_document_lines(document).join("\n");
    output.push('\n');
    output.push_str(&render_toc_lines(&document.parsed().toc()).join("\n"));
    output.push('\n');

    output
}

pub fn render_document_lines(document: &Document) -> Vec<String> {
    let mut output = String::new();

    for block in &document.parsed().blocks {
        render_block(block, &mut output);
        output.push('\n');
    }

    output.lines().map(ToOwned::to_owned).collect()
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
        Block::Rule => output.push_str("---\n"),
    }
}

fn render_heading(level: HeadingLevel, text: &str, output: &mut String) {
    output.push_str(&"#".repeat(usize::from(level.as_depth())));
    output.push(' ');
    output.push_str(text);
    output.push('\n');
}

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

#[cfg(test)]
mod tests {
    use paperview_core::Document;

    use super::render_document;

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
}
