use iced::{
    Element, Fill,
    widget::{Column, column, container, rule, scrollable, text},
};
use paperview_core::{
    Document,
    parser::{Block, HeadingLevel},
};

use crate::theme;

pub const ACTIVE_READER_SCROLLABLE_ID: &str = "active-reader-scrollable";
const BLOCK_SPACING: f32 = 18.0;
const HEADING_LINE_CHARS: usize = 36;
const BODY_LINE_CHARS: usize = 72;
const CODE_LINE_HEIGHT: f32 = 18.0;
const BODY_LINE_HEIGHT: f32 = 24.0;

pub fn view<Message: 'static>(document: &Document) -> Element<'_, Message> {
    view_with_scroll(document, None::<fn(f32) -> Message>)
}

pub fn view_with_scroll<'a, Message: 'static>(
    document: &'a Document,
    on_scroll: Option<impl Fn(f32) -> Message + 'a>,
) -> Element<'a, Message> {
    let mut content = column![].spacing(BLOCK_SPACING).width(Fill);

    for block in &document.parsed().blocks {
        content = content.push(block_view(block));
    }

    let mut scrollable = scrollable(
        container(content)
            .padding([48, 56])
            .max_width(860)
            .style(|_| theme::paper_container()),
    );

    if let Some(on_scroll) = on_scroll {
        scrollable = scrollable
            .id(ACTIVE_READER_SCROLLABLE_ID)
            .on_scroll(move |viewport| {
                let offset = viewport.relative_offset().y;
                on_scroll(if offset.is_finite() { offset } else { 0.0 })
            });
    }

    container(scrollable)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .padding([28, 0])
        .style(|_| theme::reader_backdrop())
        .into()
}

pub fn active_heading_for_scroll(
    document: &paperview_core::parser::ParsedDocument,
    progress: f32,
) -> Option<usize> {
    let toc = document.toc();

    if toc.is_empty() {
        return None;
    }

    let target_offset = scrollable_extent(document) * normalized_progress(progress);

    toc.iter()
        .take_while(|item| block_top_offset(document, item.block_index) <= target_offset)
        .last()
        .or_else(|| toc.first())
        .map(|item| item.block_index)
}

pub fn heading_scroll_progress(
    document: &paperview_core::parser::ParsedDocument,
    block_index: usize,
) -> f32 {
    let extent = scrollable_extent(document);

    if extent <= f32::EPSILON {
        return 0.0;
    }

    (block_top_offset(document, block_index) / extent).clamp(0.0, 1.0)
}

fn scrollable_extent(document: &paperview_core::parser::ParsedDocument) -> f32 {
    total_content_height(document).max(1.0)
}

fn block_top_offset(document: &paperview_core::parser::ParsedDocument, block_index: usize) -> f32 {
    document
        .blocks
        .iter()
        .take(block_index.min(document.blocks.len()))
        .enumerate()
        .map(|(index, block)| estimated_block_height(block) + spacing_after(index, document))
        .sum()
}

fn total_content_height(document: &paperview_core::parser::ParsedDocument) -> f32 {
    document
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| estimated_block_height(block) + spacing_after(index, document))
        .sum()
}

fn spacing_after(index: usize, document: &paperview_core::parser::ParsedDocument) -> f32 {
    if index + 1 < document.blocks.len() {
        BLOCK_SPACING
    } else {
        0.0
    }
}

fn estimated_block_height(block: &Block) -> f32 {
    match block {
        Block::Heading { level, text } => {
            heading_line_height(*level) * estimated_line_count(text, HEADING_LINE_CHARS)
        }
        Block::Paragraph(text) => BODY_LINE_HEIGHT * estimated_line_count(text, BODY_LINE_CHARS),
        Block::BlockQuote(text) => {
            BODY_LINE_HEIGHT * estimated_line_count(text, BODY_LINE_CHARS) + 16.0
        }
        Block::CodeBlock { code, .. } => {
            let code_lines = code.lines().count().max(1) as f32;
            12.0 + 8.0 + (CODE_LINE_HEIGHT * code_lines) + 32.0
        }
        Block::Math { source, .. } => {
            let math_lines = source.lines().count().max(1) as f32;
            12.0 + 8.0 + (CODE_LINE_HEIGHT * math_lines) + 32.0
        }
        Block::List { items, .. } => {
            let item_lines = items
                .iter()
                .map(|item| estimated_line_count(item, BODY_LINE_CHARS))
                .sum::<f32>();

            (BODY_LINE_HEIGHT * item_lines) + (8.0 * items.len().saturating_sub(1) as f32)
        }
        Block::Rule => 20.0,
    }
}

fn heading_line_height(level: HeadingLevel) -> f32 {
    match level {
        HeadingLevel::H1 => 40.0,
        HeadingLevel::H2 => 32.0,
        HeadingLevel::H3 => 28.0,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 26.0,
    }
}

fn estimated_line_count(text: &str, line_chars: usize) -> f32 {
    (text.chars().count().max(1) as f32 / line_chars as f32).ceil()
}

fn normalized_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn block_view<Message: 'static>(block: &Block) -> Element<'_, Message> {
    match block {
        Block::Heading { level, text } => heading(*level, text),
        Block::Paragraph(text) => paragraph(text),
        Block::BlockQuote(text) => blockquote(text),
        Block::CodeBlock { language, code } => code_block(language.as_deref(), code),
        Block::List { ordered, items } => list(*ordered, items),
        Block::Math { display, source } => math_block(*display, source),
        Block::Rule => rule::horizontal(1).into(),
    }
}

fn heading<Message: 'static>(level: HeadingLevel, value: &str) -> Element<'_, Message> {
    let size = match level {
        HeadingLevel::H1 => 32,
        HeadingLevel::H2 => 24,
        HeadingLevel::H3 => 20,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 18,
    };

    text(value).size(size).color(theme::READER_TEXT).into()
}

fn paragraph<Message: 'static>(value: &str) -> Element<'_, Message> {
    text(value).size(16).color(theme::READER_TEXT).into()
}

fn blockquote<Message: 'static>(value: &str) -> Element<'_, Message> {
    container(text(value).size(16).color(theme::READER_TEXT_MUTED))
        .padding([8, 14])
        .width(Fill)
        .style(|_| theme::quote_container())
        .into()
}

fn code_block<'a, Message: 'static>(
    language: Option<&'a str>,
    code: &'a str,
) -> Element<'a, Message> {
    let label = language.unwrap_or("plain");

    container(
        column![
            text(label).size(12).color(theme::SHELL_ACCENT),
            text(code).size(14).color(theme::READER_TEXT)
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Fill)
    .style(|_| theme::code_container())
    .into()
}

fn math_block<Message: 'static>(display: bool, source: &str) -> Element<'_, Message> {
    let label = if display {
        "display math"
    } else {
        "inline math"
    };

    container(
        column![
            text(label).size(12).color(theme::SHELL_ACCENT),
            text(source).size(16).color(theme::READER_TEXT)
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Fill)
    .style(|_| theme::math_container())
    .into()
}

fn list<Message: 'static>(ordered: bool, items: &[String]) -> Element<'_, Message> {
    let mut list = Column::new().spacing(8);

    for (index, item) in items.iter().enumerate() {
        let marker = if ordered {
            format!("{}.", index + 1)
        } else {
            "-".to_owned()
        };

        list = list.push(
            text(format!("{marker} {item}"))
                .size(16)
                .color(theme::READER_TEXT),
        );
    }

    list.into()
}

#[cfg(test)]
mod tests {
    use paperview_core::parser::parse_markdown;

    use super::{active_heading_for_scroll, heading_scroll_progress};

    #[test]
    fn weighted_scroll_mapping_accounts_for_large_sections() {
        let long_section = "A long section. ".repeat(80);
        let parsed = parse_markdown(&format!("# First\n\n{long_section}\n\n## Second\n\nShort."));

        assert_eq!(active_heading_for_scroll(&parsed, 0.5), Some(0));
        assert_eq!(active_heading_for_scroll(&parsed, 0.9), Some(2));
    }

    #[test]
    fn heading_scroll_progress_uses_estimated_reader_geometry() {
        let long_section = "A long section. ".repeat(80);
        let parsed = parse_markdown(&format!("# First\n\n{long_section}\n\n## Second\n\nShort."));

        assert_eq!(heading_scroll_progress(&parsed, 0), 0.0);
        assert!(heading_scroll_progress(&parsed, 2) > 0.8);
        assert_eq!(heading_scroll_progress(&parsed, usize::MAX), 1.0);
    }
}
