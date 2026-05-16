use paperview_core::{
    Document,
    parser::{Block, HeadingLevel},
};

pub fn render_document(document: &Document) -> String {
    let mut output = String::new();

    for block in &document.parsed().blocks {
        render_block(block, &mut output);
        output.push('\n');
    }

    output
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

#[cfg(test)]
mod tests {
    use paperview_core::Document;

    use super::render_document;

    #[test]
    fn renders_basic_markdown_blocks() {
        let document = Document::from_source("# PaperView\n\n- Fast\n- Native\n\n---");

        assert_eq!(
            render_document(&document),
            "# PaperView\n\n- Fast\n- Native\n\n---\n\n"
        );
    }
}
