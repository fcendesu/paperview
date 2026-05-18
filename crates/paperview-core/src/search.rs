#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub line_index: usize,
    pub column: usize,
    pub line: String,
}

#[must_use]
pub fn search_lines(source: &str, query: &str) -> Vec<SearchMatch> {
    let needle = query.trim();
    if needle.is_empty() {
        return Vec::new();
    }

    let needle = needle.to_lowercase();

    source
        .lines()
        .enumerate()
        .filter_map(|(line_index, line)| {
            let lower_line = line.to_lowercase();
            lower_line.find(&needle).map(|byte_index| SearchMatch {
                line_index,
                column: line[..byte_index].chars().count(),
                line: line.to_owned(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::search_lines;

    #[test]
    fn finds_case_insensitive_line_matches() {
        let matches = search_lines("# PaperView\n\nNative paper reader.", "paper");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[0].column, 2);
        assert_eq!(matches[1].line_index, 2);
        assert_eq!(matches[1].column, 7);
    }

    #[test]
    fn ignores_empty_queries() {
        assert!(search_lines("PaperView", " ").is_empty());
    }
}
