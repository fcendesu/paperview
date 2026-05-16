pub mod elements;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDocument {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(String),
}

#[must_use]
pub fn parse_plaintext(source: &str) -> ParsedDocument {
    let blocks = source
        .split("\n\n")
        .map(str::trim)
        .filter(|block| !block.is_empty())
        .map(|block| Block::Paragraph(block.to_owned()))
        .collect();

    ParsedDocument { blocks }
}

#[cfg(test)]
mod tests {
    use super::{Block, parse_plaintext};

    #[test]
    fn splits_plaintext_into_paragraph_blocks() {
        let parsed = parse_plaintext("First paragraph.\n\nSecond paragraph.");

        assert_eq!(
            parsed.blocks,
            vec![
                Block::Paragraph("First paragraph.".to_owned()),
                Block::Paragraph("Second paragraph.".to_owned())
            ]
        );
    }
}
