use paperview_core::{Document, parser::Block};

pub fn render_document_summary(document: &Document) -> String {
    let path = document
        .path()
        .map_or_else(|| "<memory>".to_owned(), |path| path.display().to_string());

    let block_count = document.parsed().blocks.len();
    let mut lines = vec![
        format!("PaperView GUI preview: {}", document.title()),
        format!("Path: {path}"),
        format!("Blocks: {block_count}"),
    ];

    for block in document.parsed().blocks.iter().take(5) {
        lines.push(format!("- {}", summarize_block(block)));
    }

    lines.join("\n")
}

fn summarize_block(block: &Block) -> String {
    match block {
        Block::Heading { level, text } => format!("H{} {text}", level.as_depth()),
        Block::Paragraph(text) => format!("Paragraph: {text}"),
        Block::BlockQuote(text) => format!("Quote: {text}"),
        Block::CodeBlock { language, code } => {
            let language = language.as_deref().unwrap_or("plain");
            format!("Code ({language}, {} bytes)", code.len())
        }
        Block::List { ordered, items } => {
            let kind = if *ordered { "Ordered" } else { "Unordered" };
            format!("{kind} list ({} items)", items.len())
        }
        Block::Rule => "Rule".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use paperview_core::Document;

    use super::render_document_summary;

    #[test]
    fn renders_loaded_document_summary() {
        let document = Document::from_source("# PaperView\n\nFast reader.");

        let summary = render_document_summary(&document);

        assert!(summary.contains("PaperView GUI preview: PaperView"));
        assert!(summary.contains("Blocks: 2"));
        assert!(summary.contains("- H1 PaperView"));
    }
}
