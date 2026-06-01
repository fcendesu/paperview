#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationDeck {
    slides: Vec<Slide>,
}

impl PresentationDeck {
    #[must_use]
    pub fn from_markdown(source: &str) -> Self {
        let (slides, has_rules) = split_on_rules(source);
        if has_rules {
            return Self { slides };
        }

        let slides = split_on_top_level_headings(source);
        Self { slides }
    }

    #[must_use]
    pub fn slides(&self) -> &[Slide] {
        &self.slides
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slides.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slides.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slide {
    title: String,
    source: String,
}

impl Slide {
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

#[must_use]
pub fn presentation_deck(source: &str) -> PresentationDeck {
    PresentationDeck::from_markdown(source)
}

fn split_on_rules(source: &str) -> (Vec<Slide>, bool) {
    split_source(source, is_slide_rule)
}

fn split_on_top_level_headings(source: &str) -> Vec<Slide> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in source.split_inclusive('\n') {
        if is_top_level_heading(line) && !current.trim().is_empty() {
            chunks.push(current.trim().to_owned());
            current.clear();
        }
        current.push_str(line);
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_owned());
    }

    if chunks.is_empty() {
        return Vec::new();
    }

    chunks
        .into_iter()
        .map(|source| slide_from_source(&source))
        .collect()
}

fn split_source(source: &str, is_separator: impl Fn(&str) -> bool) -> (Vec<Slide>, bool) {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut has_separator = false;

    for line in source.split_inclusive('\n') {
        if is_separator(line) {
            has_separator = true;
            if !current.trim().is_empty() {
                chunks.push(current.trim().to_owned());
                current.clear();
            }
        } else {
            current.push_str(line);
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_owned());
    }

    (
        chunks
            .into_iter()
            .map(|source| slide_from_source(&source))
            .collect(),
        has_separator,
    )
}

fn slide_from_source(source: &str) -> Slide {
    Slide {
        title: slide_title(source),
        source: source.to_owned(),
    }
}

fn slide_title(source: &str) -> String {
    source
        .lines()
        .find_map(markdown_heading_title)
        .or_else(|| {
            source
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_owned())
        })
        .unwrap_or_else(|| "Untitled Slide".to_owned())
}

fn markdown_heading_title(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let marker_count = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&marker_count) {
        return None;
    }

    let after_markers = &trimmed[marker_count..];
    after_markers
        .starts_with(char::is_whitespace)
        .then(|| after_markers.trim().trim_end_matches('#').trim().to_owned())
        .filter(|title| !title.is_empty())
}

fn is_top_level_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("# ") || trimmed.starts_with("#\t")
}

fn is_slide_rule(line: &str) -> bool {
    let trimmed = line.trim();
    matches!(trimmed, "---" | "***" | "___")
}

#[cfg(test)]
mod tests {
    use super::presentation_deck;

    #[test]
    fn splits_slides_on_thematic_rules() {
        let deck = presentation_deck("# Intro\n\nWelcome\n\n---\n\n## Details\n\nMore");

        assert_eq!(deck.len(), 2);
        assert_eq!(deck.slides()[0].title(), "Intro");
        assert_eq!(deck.slides()[0].source(), "# Intro\n\nWelcome");
        assert_eq!(deck.slides()[1].title(), "Details");
        assert_eq!(deck.slides()[1].source(), "## Details\n\nMore");
    }

    #[test]
    fn falls_back_to_top_level_headings_without_rules() {
        let deck = presentation_deck("# First\n\nOne\n\n# Second\n\nTwo\n\n## Detail\n\nMore");

        assert_eq!(deck.len(), 2);
        assert_eq!(deck.slides()[0].source(), "# First\n\nOne");
        assert_eq!(
            deck.slides()[1].source(),
            "# Second\n\nTwo\n\n## Detail\n\nMore"
        );
    }

    #[test]
    fn keeps_plain_document_as_one_slide() {
        let deck = presentation_deck("Plain notes\n\nwith context.");

        assert_eq!(deck.len(), 1);
        assert_eq!(deck.slides()[0].title(), "Plain notes");
        assert_eq!(deck.slides()[0].source(), "Plain notes\n\nwith context.");
    }

    #[test]
    fn ignores_empty_rule_chunks() {
        let deck = presentation_deck("---\n\n# Only\n\n---\n\n");

        assert_eq!(deck.len(), 1);
        assert_eq!(deck.slides()[0].title(), "Only");
    }

    #[test]
    fn empty_source_has_no_slides() {
        let deck = presentation_deck("\n\n");

        assert!(deck.is_empty());
    }
}
