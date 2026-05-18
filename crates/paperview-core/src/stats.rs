use crate::parser::{ParsedDocument, TocItem};

const WORDS_PER_MINUTE: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentStats {
    pub word_count: usize,
    pub line_count: usize,
    pub character_count: usize,
    pub heading_count: usize,
    pub estimated_reading_minutes: usize,
    pub headings: Vec<StatsHeading>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsHeading {
    pub depth: u8,
    pub title: String,
}

#[must_use]
pub fn document_stats(source: &str, parsed: &ParsedDocument) -> DocumentStats {
    let word_count = word_count(source);
    let headings = parsed
        .toc()
        .iter()
        .map(StatsHeading::from)
        .collect::<Vec<_>>();

    DocumentStats {
        word_count,
        line_count: source.lines().count(),
        character_count: source.chars().count(),
        heading_count: headings.len(),
        estimated_reading_minutes: estimated_reading_minutes(word_count),
        headings,
    }
}

fn word_count(source: &str) -> usize {
    source
        .split(|character: char| !character.is_alphanumeric() && character != '\'')
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .count()
}

fn estimated_reading_minutes(word_count: usize) -> usize {
    if word_count == 0 {
        0
    } else {
        word_count.div_ceil(WORDS_PER_MINUTE)
    }
}

impl From<&TocItem> for StatsHeading {
    fn from(item: &TocItem) -> Self {
        Self {
            depth: item.level.as_depth(),
            title: item.title.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, stats::document_stats};

    #[test]
    fn counts_document_metadata() {
        let document = Document::from_source("# PaperView\n\nNative Markdown viewer.");
        let stats = document.stats();

        assert_eq!(stats.word_count, 4);
        assert_eq!(stats.line_count, 3);
        assert_eq!(stats.heading_count, 1);
        assert_eq!(stats.estimated_reading_minutes, 1);
        assert_eq!(stats.headings[0].title, "PaperView");
    }

    #[test]
    fn empty_documents_have_zero_reading_time() {
        let document = Document::from_source("");
        let stats = document_stats(document.source(), document.parsed());

        assert_eq!(stats.word_count, 0);
        assert_eq!(stats.estimated_reading_minutes, 0);
        assert!(stats.headings.is_empty());
    }
}
