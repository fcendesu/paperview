use std::ops::Range;

use iced::{
    Color, Font,
    advanced::text::highlighter::{Format, Highlighter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownHighlight {
    HeadingMarker,
    ListMarker,
    QuoteMarker,
    Code,
    Link,
    Emphasis,
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownHighlighter {
    current_line: usize,
}

impl Highlighter for MarkdownHighlighter {
    type Settings = ();
    type Highlight = MarkdownHighlight;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self::default()
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line = self.current_line.saturating_add(1);
        markdown_highlights(line).into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

pub fn markdown_highlight_format(
    highlight: &MarkdownHighlight,
    _theme: &iced::Theme,
) -> Format<Font> {
    let color = match highlight {
        MarkdownHighlight::HeadingMarker => Color::from_rgb(0.345, 0.651, 1.0),
        MarkdownHighlight::ListMarker | MarkdownHighlight::QuoteMarker => {
            Color::from_rgb(0.545, 0.58, 0.62)
        }
        MarkdownHighlight::Code => Color::from_rgb(0.663, 0.816, 0.573),
        MarkdownHighlight::Link => Color::from_rgb(0.345, 0.651, 1.0),
        MarkdownHighlight::Emphasis => Color::from_rgb(0.906, 0.682, 0.314),
    };

    Format {
        color: Some(color),
        font: None,
    }
}

pub fn markdown_highlights(line: &str) -> Vec<(Range<usize>, MarkdownHighlight)> {
    let mut ranges = Vec::new();
    let trimmed_start = line.len() - line.trim_start().len();
    let trimmed = &line[trimmed_start..];

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        ranges.push((trimmed_start..line.len(), MarkdownHighlight::Code));
        return ranges;
    }

    if let Some(count) = heading_marker_len(trimmed) {
        ranges.push((
            trimmed_start..trimmed_start + count,
            MarkdownHighlight::HeadingMarker,
        ));
    } else if let Some(count) = list_marker_len(trimmed) {
        ranges.push((
            trimmed_start..trimmed_start + count,
            MarkdownHighlight::ListMarker,
        ));
    } else if trimmed.starts_with('>') {
        ranges.push((
            trimmed_start..trimmed_start + 1,
            MarkdownHighlight::QuoteMarker,
        ));
    }

    ranges.extend(inline_code_ranges(line).map(|range| (range, MarkdownHighlight::Code)));
    ranges.extend(link_ranges(line).map(|range| (range, MarkdownHighlight::Link)));
    ranges.extend(emphasis_marker_ranges(line).map(|range| (range, MarkdownHighlight::Emphasis)));

    ranges
}

fn heading_marker_len(line: &str) -> Option<usize> {
    let count = line.bytes().take_while(|byte| *byte == b'#').count();
    (count > 0 && count <= 6 && line.as_bytes().get(count) == Some(&b' ')).then_some(count)
}

fn list_marker_len(line: &str) -> Option<usize> {
    if matches!(line.as_bytes(), [b'-' | b'*' | b'+', b' ', ..]) {
        return Some(1);
    }

    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    (digits > 0 && line.as_bytes().get(digits..digits + 2) == Some(b". ")).then_some(digits + 1)
}

fn inline_code_ranges(line: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    paired_marker_ranges(line, "`")
}

fn emphasis_marker_ranges(line: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    paired_marker_ranges(line, "**").chain(paired_marker_ranges(line, "*"))
}

fn paired_marker_ranges<'a>(
    line: &'a str,
    marker: &'static str,
) -> impl Iterator<Item = Range<usize>> + 'a {
    let mut search_start = 0;
    std::iter::from_fn(move || {
        let start = line[search_start..].find(marker)? + search_start;
        let end = line[start + marker.len()..].find(marker)? + start + marker.len();
        search_start = end + marker.len();
        Some(start..start + marker.len())
    })
}

fn link_ranges(line: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let mut search_start = 0;
    std::iter::from_fn(move || {
        let label_start = line[search_start..].find('[')? + search_start;
        let label_end = line[label_start..].find("](")? + label_start;
        let target_end = line[label_end + 2..].find(')')? + label_end + 2;
        search_start = target_end + 1;
        Some(label_start..target_end + 1)
    })
}

#[cfg(test)]
mod tests {
    use super::{MarkdownHighlight, markdown_highlights};

    #[test]
    fn highlights_block_markers() {
        let highlights = markdown_highlights("## Heading");

        assert_eq!(highlights[0].0, 0..2);
        assert_eq!(highlights[0].1, MarkdownHighlight::HeadingMarker);

        let highlights = markdown_highlights("- Item");
        assert_eq!(highlights[0].1, MarkdownHighlight::ListMarker);

        let highlights = markdown_highlights("> Quote");
        assert_eq!(highlights[0].1, MarkdownHighlight::QuoteMarker);
    }

    #[test]
    fn highlights_inline_ranges() {
        let highlights = markdown_highlights("A **bold** [link](docs.md) and `code`");

        assert!(
            highlights
                .iter()
                .any(|(_, highlight)| *highlight == MarkdownHighlight::Emphasis)
        );
        assert!(
            highlights
                .iter()
                .any(|(_, highlight)| *highlight == MarkdownHighlight::Link)
        );
        assert!(
            highlights
                .iter()
                .any(|(_, highlight)| *highlight == MarkdownHighlight::Code)
        );
    }
}
