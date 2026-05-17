use iced::{
    Background, Element, Fill, Font, font,
    widget::{Column, Row, column, container, rich_text, rule, scrollable, span, text},
};
use paperview_core::{
    Document,
    parser::{Block, HeadingLevel, InlineSpan, TableAlignment, elements::inline},
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
        Block::Paragraph(spans) => {
            BODY_LINE_HEIGHT * estimated_line_count(&inline::plain_text(spans), BODY_LINE_CHARS)
        }
        Block::BlockQuote(text) => {
            BODY_LINE_HEIGHT * estimated_line_count(text, BODY_LINE_CHARS) + 16.0
        }
        Block::CodeBlock { code, .. } => {
            let code_lines = code.lines().count().max(1) as f32;
            12.0 + 8.0 + (CODE_LINE_HEIGHT * code_lines) + 32.0
        }
        Block::Diagram { source, .. } => {
            let diagram_lines = source.lines().count().max(1) as f32;
            12.0 + 8.0 + (CODE_LINE_HEIGHT * diagram_lines) + 32.0
        }
        Block::Image { alt, url, title } => {
            let text_lines = estimated_line_count(alt, BODY_LINE_CHARS)
                + estimated_line_count(url, BODY_LINE_CHARS)
                + if title.is_empty() { 0.0 } else { 1.0 };
            24.0 + (BODY_LINE_HEIGHT * text_lines)
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
        Block::Table { header, rows, .. } => {
            let row_count = rows.len() + usize::from(!header.is_empty());
            24.0 + (BODY_LINE_HEIGHT * row_count.max(1) as f32)
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
        Block::Paragraph(spans) => paragraph(spans),
        Block::BlockQuote(text) => blockquote(text),
        Block::CodeBlock { language, code } => code_block(language.as_deref(), code),
        Block::Diagram { language, source } => diagram_block(language, source),
        Block::Image { alt, url, title } => image_block(alt, url, title),
        Block::List { ordered, items } => list(*ordered, items),
        Block::Math { display, source } => math_block(*display, source),
        Block::Table {
            alignments,
            header,
            rows,
        } => table_block(alignments, header, rows),
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

fn paragraph<Message: 'static>(spans: &[InlineSpan]) -> Element<'_, Message> {
    let spans = spans.iter().map(rich_span).collect::<Vec<_>>();

    rich_text(spans).size(16).into()
}

fn rich_span(source: &InlineSpan) -> iced::widget::text::Span<'_, (), Font> {
    let mut output = span(source.text.as_str()).color(theme::READER_TEXT);

    if source.strong || source.emphasis {
        output = output.font(Font {
            weight: if source.strong {
                font::Weight::Bold
            } else {
                font::Weight::Normal
            },
            style: if source.emphasis {
                font::Style::Italic
            } else {
                font::Style::Normal
            },
            ..Font::default()
        });
    }

    if source.code {
        output = output
            .font(Font::MONOSPACE)
            .background(Background::Color(theme::CODE_BACKGROUND))
            .padding([1, 4]);
    }

    if source.link.is_some() {
        output = output.color(theme::SHELL_ACCENT).underline(true);
    }

    output
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

fn diagram_block<'a, Message: 'static>(language: &'a str, source: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(language).size(12).color(theme::SHELL_ACCENT),
            text(source).size(14).color(theme::READER_TEXT)
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Fill)
    .style(|_| theme::diagram_container())
    .into()
}

fn image_block<'a, Message: 'static>(
    alt: &'a str,
    url: &'a str,
    title: &'a str,
) -> Element<'a, Message> {
    let mut details = column![
        text("image").size(12).color(theme::SHELL_ACCENT),
        text(if alt.is_empty() {
            "Untitled image"
        } else {
            alt
        })
        .size(16)
        .color(theme::READER_TEXT),
        text(url).size(13).color(theme::READER_TEXT_MUTED)
    ]
    .spacing(6);

    if !title.is_empty() {
        details = details.push(text(title).size(13).color(theme::READER_TEXT_MUTED));
    }

    container(details)
        .padding(16)
        .width(Fill)
        .style(|_| theme::image_container())
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

fn table_block<'a, Message: 'static>(
    alignments: &'a [TableAlignment],
    header: &'a [String],
    rows: &'a [Vec<String>],
) -> Element<'a, Message> {
    let mut table = Column::new().spacing(0).width(Fill);

    if !header.is_empty() {
        table = table.push(table_row(header, alignments, true));
    }

    for row in rows {
        table = table.push(table_row(row, alignments, false));
    }

    container(table)
        .width(Fill)
        .style(|_| theme::table_container())
        .into()
}

fn table_row<'a, Message: 'static>(
    cells: &'a [String],
    alignments: &'a [TableAlignment],
    is_header: bool,
) -> Element<'a, Message> {
    let mut row = Row::new().spacing(0).width(Fill);

    for (index, cell) in cells.iter().enumerate() {
        row = row.push(table_cell(
            cell,
            alignments
                .get(index)
                .copied()
                .unwrap_or(TableAlignment::None),
            is_header,
        ));
    }

    row.into()
}

fn table_cell<Message: 'static>(
    value: &str,
    alignment: TableAlignment,
    is_header: bool,
) -> Element<'_, Message> {
    let mut label = text(value)
        .size(if is_header { 14 } else { 13 })
        .color(if is_header {
            theme::READER_TEXT
        } else {
            theme::READER_TEXT_MUTED
        });

    label = match alignment {
        TableAlignment::Right => label.align_x(iced::alignment::Horizontal::Right),
        TableAlignment::Center => label.align_x(iced::alignment::Horizontal::Center),
        TableAlignment::None | TableAlignment::Left => label,
    };

    container(label)
        .padding([8, 10])
        .width(Fill)
        .style(move |_| theme::table_cell_container(is_header))
        .into()
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
