use std::path::{Path, PathBuf};

use iced::{
    Background, ContentFit, Element, Fill, Font, Length, font,
    widget::{
        Column, Row, column, container, image as image_widget, rich_text, rule, scrollable, span,
        text,
    },
};
use paperview_core::{
    Document,
    parser::{
        Block, HeadingLevel, InlineSpan, TableAlignment, TableCell, TableRow,
        elements::{diagram, inline},
    },
};

use crate::theme;

pub const ACTIVE_READER_SCROLLABLE_ID: &str = "active-reader-scrollable";
const BLOCK_SPACING: f32 = 18.0;
const HEADING_LINE_CHARS: usize = 36;
const BODY_LINE_CHARS: usize = 72;
const CODE_LINE_HEIGHT: f32 = 18.0;
const BODY_LINE_HEIGHT: f32 = 24.0;
const IMAGE_PREVIEW_HEIGHT: f32 = 360.0;

pub fn view<Message: 'static>(
    document: &Document,
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    view_with_scroll(document, None::<fn(f32) -> Message>, on_link_click)
}

pub fn view_with_scroll<'a, Message: 'static>(
    document: &'a Document,
    on_scroll: Option<impl Fn(f32) -> Message + 'a>,
    on_link_click: fn(String) -> Message,
) -> Element<'a, Message> {
    let mut content = column![].spacing(BLOCK_SPACING).width(Fill);
    let document_path = document.path().map(PathBuf::as_path);

    for block in &document.parsed().blocks {
        content = content.push(block_view(block, document_path, on_link_click));
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
        Block::Heading { level, spans } => {
            heading_line_height(*level)
                * estimated_line_count(&inline::plain_text(spans), HEADING_LINE_CHARS)
        }
        Block::Paragraph(spans) => {
            BODY_LINE_HEIGHT * estimated_line_count(&inline::plain_text(spans), BODY_LINE_CHARS)
        }
        Block::BlockQuote(spans) => {
            BODY_LINE_HEIGHT * estimated_line_count(&inline::plain_text(spans), BODY_LINE_CHARS)
                + 16.0
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
                .map(|item| estimated_line_count(&inline::plain_text(item), BODY_LINE_CHARS))
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

fn block_view<'a, Message: 'static>(
    block: &'a Block,
    document_path: Option<&'a Path>,
    on_link_click: fn(String) -> Message,
) -> Element<'a, Message> {
    match block {
        Block::Heading { level, spans } => heading(*level, spans, on_link_click),
        Block::Paragraph(spans) => paragraph(spans, on_link_click),
        Block::BlockQuote(spans) => blockquote(spans, on_link_click),
        Block::CodeBlock { language, code } => code_block(language.as_deref(), code),
        Block::Diagram { language, source } => diagram_block(language, source),
        Block::Image { alt, url, title } => image_block(alt, url, title, document_path),
        Block::List { ordered, items } => list(*ordered, items, on_link_click),
        Block::Math { display, source } => math_block(*display, source),
        Block::Table {
            alignments,
            header,
            rows,
        } => table_block(alignments, header, rows, on_link_click),
        Block::Rule => rule::horizontal(1).into(),
    }
}

fn heading<Message: 'static>(
    level: HeadingLevel,
    spans: &[InlineSpan],
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    let size = match level {
        HeadingLevel::H1 => 32,
        HeadingLevel::H2 => 24,
        HeadingLevel::H3 => 20,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 18,
    };

    inline_text(spans, size, theme::READER_TEXT, on_link_click)
}

fn paragraph<Message: 'static>(
    spans: &[InlineSpan],
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    inline_text(spans, 16, theme::READER_TEXT, on_link_click)
}

fn inline_text<Message: 'static>(
    spans: &[InlineSpan],
    size: u32,
    base_color: iced::Color,
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    let spans = spans
        .iter()
        .map(|span| rich_span(span, base_color))
        .collect::<Vec<_>>();

    rich_text(spans)
        .size(size)
        .on_link_click(on_link_click)
        .into()
}

fn rich_span(
    source: &InlineSpan,
    base_color: iced::Color,
) -> iced::widget::text::Span<'_, String, Font> {
    let mut output = span(source.text.as_str()).color(base_color);

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

    if let Some(link) = &source.link {
        output = output
            .color(theme::SHELL_ACCENT)
            .underline(true)
            .link(link.clone());
    }

    output
}

fn blockquote<Message: 'static>(
    spans: &[InlineSpan],
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    container(inline_text(
        spans,
        16,
        theme::READER_TEXT_MUTED,
        on_link_click,
    ))
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
    let mut content = column![text(language).size(12).color(theme::SHELL_ACCENT)].spacing(12);

    if let Some(preview) = diagram::flowchart_preview(source) {
        content = content.push(flowchart_preview(preview));
    }

    content = content.push(text(source).size(14).color(theme::READER_TEXT));

    container(content)
        .padding(16)
        .width(Fill)
        .style(|_| theme::diagram_container())
        .into()
}

fn flowchart_preview<Message: 'static>(
    preview: diagram::FlowchartPreview,
) -> Element<'static, Message> {
    let direction = match preview.direction {
        diagram::FlowchartDirection::TopDown => "top down",
        diagram::FlowchartDirection::BottomTop => "bottom to top",
        diagram::FlowchartDirection::LeftRight => "left to right",
        diagram::FlowchartDirection::RightLeft => "right to left",
    };

    let mut rows = Column::new().spacing(8).push(
        text(format!("flowchart preview - {direction}"))
            .size(13)
            .color(theme::READER_TEXT_MUTED),
    );

    for edge in preview.edges {
        rows = rows.push(flowchart_edge(edge));
    }

    container(rows)
        .padding(12)
        .width(Fill)
        .style(|_| theme::paper_container())
        .into()
}

fn flowchart_edge<Message: 'static>(edge: diagram::FlowchartEdge) -> Element<'static, Message> {
    Row::new()
        .spacing(8)
        .push(flowchart_node(edge.from))
        .push(text("->").size(14).color(theme::SHELL_ACCENT))
        .push(flowchart_node(edge.to))
        .into()
}

fn flowchart_node<Message: 'static>(label: String) -> Element<'static, Message> {
    container(text(label).size(13).color(theme::READER_TEXT))
        .padding([6, 10])
        .style(|_| theme::table_cell_container(false))
        .into()
}

fn image_block<'a, Message: 'static>(
    alt: &'a str,
    url: &'a str,
    title: &'a str,
    document_path: Option<&'a Path>,
) -> Element<'a, Message> {
    let resolved_path = resolve_image_path(url, document_path);
    let display_path = resolved_path
        .as_ref()
        .map_or_else(|| url.to_owned(), |path| path.display().to_string());

    let mut details = column![
        text("image").size(12).color(theme::SHELL_ACCENT),
        text(if alt.is_empty() {
            "Untitled image"
        } else {
            alt
        })
        .size(16)
        .color(theme::READER_TEXT),
        text(display_path).size(13).color(theme::READER_TEXT_MUTED)
    ]
    .spacing(6);

    if !title.is_empty() {
        details = details.push(text(title).size(13).color(theme::READER_TEXT_MUTED));
    }

    if let Some(path) = resolved_path {
        details = details.push(
            image_widget(path)
                .width(Length::Fill)
                .height(IMAGE_PREVIEW_HEIGHT)
                .content_fit(ContentFit::Contain),
        );
    }

    container(details)
        .padding(16)
        .width(Fill)
        .style(|_| theme::image_container())
        .into()
}

fn resolve_image_path(url: &str, document_path: Option<&Path>) -> Option<PathBuf> {
    if url.trim().is_empty() || is_remote_image_url(url) {
        return None;
    }

    let path = PathBuf::from(url);
    let resolved = if path.is_absolute() {
        path
    } else {
        document_path.and_then(Path::parent)?.join(path)
    };

    resolved.is_file().then_some(resolved)
}

fn is_remote_image_url(url: &str) -> bool {
    url.contains("://") || url.starts_with("data:")
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

fn list<Message: 'static>(
    ordered: bool,
    items: &[Vec<InlineSpan>],
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    let mut list = Column::new().spacing(8);

    for (index, item) in items.iter().enumerate() {
        let marker = if ordered {
            format!("{}.", index + 1)
        } else {
            "-".to_owned()
        };

        list = list.push(
            Row::new()
                .spacing(4)
                .push(text(marker).size(16).color(theme::READER_TEXT))
                .push(inline_text(item, 16, theme::READER_TEXT, on_link_click)),
        );
    }

    list.into()
}

fn table_block<'a, Message: 'static>(
    alignments: &'a [TableAlignment],
    header: &'a TableRow,
    rows: &'a [TableRow],
    on_link_click: fn(String) -> Message,
) -> Element<'a, Message> {
    let mut table = Column::new().spacing(0).width(Fill);

    if !header.is_empty() {
        table = table.push(table_row(header, alignments, true, on_link_click));
    }

    for row in rows {
        table = table.push(table_row(row, alignments, false, on_link_click));
    }

    container(table)
        .width(Fill)
        .style(|_| theme::table_container())
        .into()
}

fn table_row<'a, Message: 'static>(
    cells: &'a [TableCell],
    alignments: &'a [TableAlignment],
    is_header: bool,
    on_link_click: fn(String) -> Message,
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
            on_link_click,
        ));
    }

    row.into()
}

fn table_cell<Message: 'static>(
    value: &[InlineSpan],
    alignment: TableAlignment,
    is_header: bool,
    on_link_click: fn(String) -> Message,
) -> Element<'_, Message> {
    let label = inline_text(
        value,
        if is_header { 14 } else { 13 },
        if is_header {
            theme::READER_TEXT
        } else {
            theme::READER_TEXT_MUTED
        },
        on_link_click,
    );

    let mut cell = container(label)
        .padding([8, 10])
        .width(Fill)
        .style(move |_| theme::table_cell_container(is_header));

    cell = match alignment {
        TableAlignment::Right => cell.align_x(iced::alignment::Horizontal::Right),
        TableAlignment::Center => cell.align_x(iced::alignment::Horizontal::Center),
        TableAlignment::None | TableAlignment::Left => cell,
    };

    cell.into()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use paperview_core::parser::parse_markdown;

    use super::{active_heading_for_scroll, heading_scroll_progress, resolve_image_path};

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

    #[test]
    fn relative_image_paths_resolve_from_document_parent() {
        let dir = temp_dir("image-resolve");
        let image_dir = dir.join("assets");
        let image_path = image_dir.join("preview.png");
        let document_path = dir.join("doc.md");

        fs::create_dir_all(&image_dir).expect("create image dir");
        fs::write(&image_path, b"not decoded in this test").expect("write image placeholder");
        fs::write(&document_path, "# Image").expect("write document");

        assert_eq!(
            resolve_image_path("assets/preview.png", Some(&document_path)),
            Some(image_path.clone())
        );
        assert_eq!(
            resolve_image_path("assets/missing.png", Some(&document_path)),
            None
        );

        fs::remove_dir_all(dir).expect("remove temp dir");
    }

    #[test]
    fn remote_image_urls_do_not_resolve_to_local_previews() {
        assert_eq!(
            resolve_image_path("https://example.com/image.png", None),
            None
        );
        assert_eq!(resolve_image_path("data:image/png;base64,AAAA", None), None);
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("paperview-reader-{nanos}-{name}"))
    }
}
