use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use iced::{
    Background, ContentFit, Element, Fill, Font, Length, font,
    widget::{
        Column, Row, button, column, container, image as image_widget, rich_text, rule, scrollable,
        span, text,
    },
};
use paperview_core::{
    Document,
    parser::{
        Block, HeadingLevel, InlineSpan, ListItem, TableAlignment, TableCell, TableRow,
        elements::{diagram, inline, math},
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
const TABLE_MIN_COLUMN_PORTION: u16 = 8;
const TABLE_MAX_COLUMN_PORTION: u16 = 32;

#[derive(Debug, Clone)]
pub enum RemoteImage {
    Loading,
    Loaded(Vec<u8>),
    Failed(String),
}

pub fn view_with_remote_images<'a, Message: Clone + 'static>(
    document: &'a Document,
    on_link_click: fn(String) -> Message,
    on_task_toggle: Option<fn(usize) -> Message>,
    remote_images: &'a HashMap<String, RemoteImage>,
) -> Element<'a, Message> {
    view_with_scroll_and_search(
        document,
        None::<fn(f32) -> Message>,
        on_link_click,
        None,
        None,
        on_task_toggle,
        Some(remote_images),
    )
}

pub fn view_with_search_and_remote_images<'a, Message: Clone + 'static>(
    document: &'a Document,
    on_scroll: Option<impl Fn(f32) -> Message + 'a>,
    on_link_click: fn(String) -> Message,
    search_query: Option<&'a str>,
    active_search_line: Option<&'a str>,
    on_task_toggle: Option<fn(usize) -> Message>,
    remote_images: &'a HashMap<String, RemoteImage>,
) -> Element<'a, Message> {
    view_with_scroll_and_search(
        document,
        on_scroll,
        on_link_click,
        search_query,
        active_search_line,
        on_task_toggle,
        Some(remote_images),
    )
}

fn view_with_scroll_and_search<'a, Message: Clone + 'static>(
    document: &'a Document,
    on_scroll: Option<impl Fn(f32) -> Message + 'a>,
    on_link_click: fn(String) -> Message,
    search_query: Option<&'a str>,
    active_search_line: Option<&'a str>,
    on_task_toggle: Option<fn(usize) -> Message>,
    remote_images: Option<&'a HashMap<String, RemoteImage>>,
) -> Element<'a, Message> {
    let mut content = column![].spacing(BLOCK_SPACING).width(Fill);
    let document_path = document.path().map(PathBuf::as_path);
    let search_context = SearchContext::new(search_query, active_search_line);

    for block in &document.parsed().blocks {
        content = content.push(block_view(
            block,
            document_path,
            on_link_click,
            render_search_context(block, search_context),
            on_task_toggle,
            remote_images,
        ));
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
                .map(|item| {
                    estimated_line_count(&inline::plain_text(&item.content), BODY_LINE_CHARS)
                })
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

fn block_view<'a, Message: Clone + 'static>(
    block: &'a Block,
    document_path: Option<&'a Path>,
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
    on_task_toggle: Option<fn(usize) -> Message>,
    remote_images: Option<&'a HashMap<String, RemoteImage>>,
) -> Element<'a, Message> {
    match block {
        Block::Heading { level, spans } => heading(*level, spans, on_link_click, search),
        Block::Paragraph(spans) => paragraph(spans, on_link_click, search),
        Block::BlockQuote(spans) => blockquote(spans, on_link_click, search),
        Block::CodeBlock { language, code } => code_block(language.as_deref(), code),
        Block::Diagram { language, source } => diagram_block(language, source),
        Block::Image { alt, url, title } => {
            image_block(alt, url, title, document_path, remote_images)
        }
        Block::List { ordered, items } => {
            list(*ordered, items, on_link_click, search, on_task_toggle)
        }
        Block::Math { display, source } => math_block(*display, source),
        Block::Table {
            alignments,
            header,
            rows,
        } => table_block(alignments, header, rows, on_link_click, search),
        Block::Rule => rule::horizontal(1).into(),
    }
}

fn heading<'a, Message: Clone + 'static>(
    level: HeadingLevel,
    spans: &'a [InlineSpan],
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    let size = match level {
        HeadingLevel::H1 => 32,
        HeadingLevel::H2 => 24,
        HeadingLevel::H3 => 20,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 18,
    };

    inline_text(spans, size, theme::READER_TEXT, on_link_click, search)
}

fn paragraph<'a, Message: Clone + 'static>(
    spans: &'a [InlineSpan],
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    inline_text(spans, 16, theme::READER_TEXT, on_link_click, search)
}

fn inline_text<'a, Message: Clone + 'static>(
    spans: &'a [InlineSpan],
    size: u32,
    base_color: iced::Color,
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    let spans = spans
        .iter()
        .flat_map(|span| rich_spans(span, base_color, search))
        .collect::<Vec<_>>();

    rich_text(spans)
        .size(size)
        .on_link_click(on_link_click)
        .into()
}

fn rich_spans<'a>(
    source: &'a InlineSpan,
    base_color: iced::Color,
    search: Option<RenderSearch<'_>>,
) -> Vec<iced::widget::text::Span<'a, String, Font>> {
    highlight_segments(&source.text, search)
        .into_iter()
        .map(|segment| rich_span_segment(source, segment.text, base_color, segment.highlight))
        .collect()
}

fn rich_span_segment<'a>(
    source: &'a InlineSpan,
    text_value: &'a str,
    base_color: iced::Color,
    highlight: SearchHighlight,
) -> iced::widget::text::Span<'a, String, Font> {
    let mut output = span(text_value).color(match highlight {
        SearchHighlight::None => base_color,
        SearchHighlight::Match => theme::SEARCH_HIGHLIGHT_TEXT,
        SearchHighlight::Active => theme::SEARCH_ACTIVE_HIGHLIGHT_TEXT,
    });

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

    if source.code || source.math {
        output = output.font(Font::MONOSPACE).padding([1, 4]);
    }

    output = if highlight == SearchHighlight::Active {
        output.background(Background::Color(theme::SEARCH_ACTIVE_HIGHLIGHT_BACKGROUND))
    } else if highlight == SearchHighlight::Match {
        output.background(Background::Color(theme::SEARCH_HIGHLIGHT_BACKGROUND))
    } else if source.math {
        output.background(Background::Color(theme::INLINE_MATH_BACKGROUND))
    } else if source.code {
        output.background(Background::Color(theme::CODE_BACKGROUND))
    } else {
        output
    };

    if let Some(link) = &source.link {
        output = output.underline(true).link(link.clone());
        if highlight == SearchHighlight::None {
            output = output.color(theme::SHELL_ACCENT);
        }
    }

    output
}

fn blockquote<'a, Message: Clone + 'static>(
    spans: &'a [InlineSpan],
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    container(inline_text(
        spans,
        16,
        theme::READER_TEXT_MUTED,
        on_link_click,
        search,
    ))
    .padding([8, 14])
    .width(Fill)
    .style(|_| theme::quote_container())
    .into()
}

fn code_block<'a, Message: Clone + 'static>(
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

fn diagram_block<'a, Message: Clone + 'static>(
    language: &'a str,
    source: &'a str,
) -> Element<'a, Message> {
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

fn flowchart_preview<Message: Clone + 'static>(
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

fn flowchart_edge<Message: Clone + 'static>(
    edge: diagram::FlowchartEdge,
) -> Element<'static, Message> {
    let mut row = Row::new()
        .spacing(8)
        .push(flowchart_node(edge.from))
        .push(text("->").size(14).color(theme::SHELL_ACCENT));

    if let Some(label) = edge.label {
        row = row.push(text(label).size(12).color(theme::READER_TEXT_MUTED));
    }

    row.push(flowchart_node(edge.to)).into()
}

fn flowchart_node<Message: Clone + 'static>(label: String) -> Element<'static, Message> {
    container(text(label).size(13).color(theme::READER_TEXT))
        .padding([6, 10])
        .style(|_| theme::table_cell_container(false))
        .into()
}

fn image_block<'a, Message: Clone + 'static>(
    alt: &'a str,
    url: &'a str,
    title: &'a str,
    document_path: Option<&'a Path>,
    remote_images: Option<&'a HashMap<String, RemoteImage>>,
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
        if let Some(dimensions) = image_dimensions_from_path(&path) {
            details = details.push(
                text(dimensions.label())
                    .size(13)
                    .color(theme::READER_TEXT_MUTED),
            );
        }

        details = details.push(
            image_widget(path)
                .width(Length::Fill)
                .height(IMAGE_PREVIEW_HEIGHT)
                .content_fit(ContentFit::Contain),
        );
    } else if is_fetchable_remote_image_url(url) {
        details = match remote_images.and_then(|images| images.get(url)) {
            Some(RemoteImage::Loaded(bytes)) => {
                if let Some(dimensions) = image_dimensions_from_bytes(bytes) {
                    details = details.push(
                        text(dimensions.label())
                            .size(13)
                            .color(theme::READER_TEXT_MUTED),
                    );
                }

                details.push(
                    image_widget(image_widget::Handle::from_bytes(bytes.clone()))
                        .width(Length::Fill)
                        .height(IMAGE_PREVIEW_HEIGHT)
                        .content_fit(ContentFit::Contain),
                )
            }
            Some(RemoteImage::Failed(error)) => details.push(
                text(format!("Remote preview unavailable: {error}"))
                    .size(13)
                    .color(theme::READER_TEXT_MUTED),
            ),
            Some(RemoteImage::Loading) | None => details.push(
                text("Loading remote preview...")
                    .size(13)
                    .color(theme::READER_TEXT_MUTED),
            ),
        };
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

pub(crate) fn is_fetchable_remote_image_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageDimensions {
    width: u32,
    height: u32,
}

impl ImageDimensions {
    fn label(self) -> String {
        format!("{} x {} px", self.width, self.height)
    }
}

fn image_dimensions_from_path(path: &Path) -> Option<ImageDimensions> {
    let bytes = fs::read(path).ok()?;
    image_dimensions_from_bytes(&bytes)
}

fn image_dimensions_from_bytes(bytes: &[u8]) -> Option<ImageDimensions> {
    png_dimensions(bytes)
        .or_else(|| gif_dimensions(bytes))
        .or_else(|| jpeg_dimensions(bytes))
        .or_else(|| webp_dimensions(bytes))
}

fn png_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }

    Some(ImageDimensions {
        width: u32::from_be_bytes(bytes[16..20].try_into().ok()?),
        height: u32::from_be_bytes(bytes[20..24].try_into().ok()?),
    })
}

fn gif_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    if bytes.len() < 10 || (&bytes[..6] != b"GIF87a" && &bytes[..6] != b"GIF89a") {
        return None;
    }

    Some(ImageDimensions {
        width: u16::from_le_bytes(bytes[6..8].try_into().ok()?).into(),
        height: u16::from_le_bytes(bytes[8..10].try_into().ok()?).into(),
    })
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }

    let mut cursor = 2;
    while cursor + 9 < bytes.len() {
        while cursor < bytes.len() && bytes[cursor] != 0xff {
            cursor += 1;
        }
        while cursor < bytes.len() && bytes[cursor] == 0xff {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }

        let marker = bytes[cursor];
        cursor += 1;

        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if matches!(marker, 0x01 | 0xd0..=0xd7) {
            continue;
        }
        if cursor + 2 > bytes.len() {
            return None;
        }

        let segment_length = usize::from(u16::from_be_bytes(
            bytes[cursor..cursor + 2].try_into().ok()?,
        ));
        if segment_length < 2 || cursor + segment_length > bytes.len() {
            return None;
        }

        if matches!(
            marker,
            0xc0 | 0xc1
                | 0xc2
                | 0xc3
                | 0xc5
                | 0xc6
                | 0xc7
                | 0xc9
                | 0xca
                | 0xcb
                | 0xcd
                | 0xce
                | 0xcf
        ) {
            if segment_length < 7 {
                return None;
            }

            return Some(ImageDimensions {
                height: u16::from_be_bytes(bytes[cursor + 3..cursor + 5].try_into().ok()?).into(),
                width: u16::from_be_bytes(bytes[cursor + 5..cursor + 7].try_into().ok()?).into(),
            });
        }

        cursor += segment_length;
    }

    None
}

fn webp_dimensions(bytes: &[u8]) -> Option<ImageDimensions> {
    if bytes.len() < 30 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }

    match &bytes[12..16] {
        b"VP8 " if bytes.len() >= 30 && &bytes[23..26] == b"\x9d\x01\x2a" => {
            Some(ImageDimensions {
                width: u32::from(u16::from_le_bytes(bytes[26..28].try_into().ok()?) & 0x3fff),
                height: u32::from(u16::from_le_bytes(bytes[28..30].try_into().ok()?) & 0x3fff),
            })
        }
        b"VP8L" if bytes.len() >= 25 && bytes[20] == 0x2f => {
            let b1 = u32::from(bytes[21]);
            let b2 = u32::from(bytes[22]);
            let b3 = u32::from(bytes[23]);
            let b4 = u32::from(bytes[24]);

            Some(ImageDimensions {
                width: 1 + (((b2 & 0x3f) << 8) | b1),
                height: 1 + (((b4 & 0x0f) << 10) | (b3 << 2) | ((b2 & 0xc0) >> 6)),
            })
        }
        b"VP8X" if bytes.len() >= 30 => Some(ImageDimensions {
            width: 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]),
            height: 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]),
        }),
        _ => None,
    }
}

fn math_block<Message: Clone + 'static>(display: bool, source: &str) -> Element<'_, Message> {
    let label = if display {
        "display math"
    } else {
        "inline math"
    };

    let mut content = column![text(label).size(12).color(theme::SHELL_ACCENT)].spacing(8);

    if let Some(preview) = math::readable_preview(source) {
        content = content.push(
            container(text(preview).size(20).color(theme::READER_TEXT))
                .padding([8, 10])
                .width(Fill)
                .style(|_| theme::paper_container()),
        );
    }

    content = content.push(text(source).size(16).color(theme::READER_TEXT));

    container(content)
        .padding(16)
        .width(Fill)
        .style(|_| theme::math_container())
        .into()
}

fn list<'a, Message: Clone + 'static>(
    ordered: bool,
    items: &'a [ListItem],
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
    on_task_toggle: Option<fn(usize) -> Message>,
) -> Element<'a, Message> {
    let mut list = Column::new().spacing(8);

    for (index, item) in items.iter().enumerate() {
        let marker = match item.checked {
            Some(true) if ordered => format!("{}. ☑", index + 1),
            Some(false) if ordered => format!("{}. ☐", index + 1),
            Some(true) => "☑".to_owned(),
            Some(false) => "☐".to_owned(),
            None if ordered => format!("{}.", index + 1),
            None => "-".to_owned(),
        };
        let marker_text = text(marker).size(16).color(theme::READER_TEXT);
        let marker: Element<'a, Message> =
            if let (Some(toggle), Some(line_index)) = (on_task_toggle, item.source_line) {
                button(marker_text)
                    .padding([1, 4])
                    .style(|_, status| theme::task_checkbox_button(status))
                    .on_press(toggle(line_index))
                    .into()
            } else {
                marker_text.into()
            };

        list = list.push(Row::new().spacing(4).push(marker).push(inline_text(
            &item.content,
            16,
            theme::READER_TEXT,
            on_link_click,
            search,
        )));
    }

    list.into()
}

fn table_block<'a, Message: Clone + 'static>(
    alignments: &'a [TableAlignment],
    header: &'a TableRow,
    rows: &'a [TableRow],
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    let mut table = Column::new().spacing(0).width(Fill);
    let column_portions = table_column_portions(header, rows);

    if !header.is_empty() {
        table = table.push(table_row(
            header,
            alignments,
            &column_portions,
            true,
            on_link_click,
            search,
        ));
    }

    for row in rows {
        table = table.push(table_row(
            row,
            alignments,
            &column_portions,
            false,
            on_link_click,
            search,
        ));
    }

    container(table)
        .width(Fill)
        .style(|_| theme::table_container())
        .into()
}

fn table_row<'a, Message: Clone + 'static>(
    cells: &'a [TableCell],
    alignments: &'a [TableAlignment],
    column_portions: &[u16],
    is_header: bool,
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    let mut row = Row::new().spacing(0).width(Fill);

    for (index, cell) in cells.iter().enumerate() {
        row = row.push(table_cell(
            cell,
            alignments
                .get(index)
                .copied()
                .unwrap_or(TableAlignment::None),
            column_portions.get(index).copied().unwrap_or(1),
            is_header,
            on_link_click,
            search,
        ));
    }

    row.into()
}

fn table_cell<'a, Message: Clone + 'static>(
    value: &'a [InlineSpan],
    alignment: TableAlignment,
    column_portion: u16,
    is_header: bool,
    on_link_click: fn(String) -> Message,
    search: Option<RenderSearch<'a>>,
) -> Element<'a, Message> {
    let label = inline_text(
        value,
        if is_header { 14 } else { 13 },
        if is_header {
            theme::READER_TEXT
        } else {
            theme::READER_TEXT_MUTED
        },
        on_link_click,
        search,
    );

    let mut cell = container(label)
        .padding([8, 10])
        .width(Length::FillPortion(column_portion))
        .style(move |_| theme::table_cell_container(is_header));

    cell = match alignment {
        TableAlignment::Right => cell.align_x(iced::alignment::Horizontal::Right),
        TableAlignment::Center => cell.align_x(iced::alignment::Horizontal::Center),
        TableAlignment::None | TableAlignment::Left => cell,
    };

    cell.into()
}

fn table_column_portions(header: &TableRow, rows: &[TableRow]) -> Vec<u16> {
    let column_count = std::iter::once(header.len())
        .chain(rows.iter().map(Vec::len))
        .max()
        .unwrap_or(0);

    (0..column_count)
        .map(|index| {
            std::iter::once(header)
                .chain(rows.iter())
                .filter_map(|row| row.get(index))
                .map(table_cell_portion)
                .max()
                .unwrap_or(TABLE_MIN_COLUMN_PORTION)
        })
        .collect()
}

fn table_cell_portion(cell: &TableCell) -> u16 {
    let text_width = inline::plain_text(cell).chars().count();

    text_width
        .clamp(
            usize::from(TABLE_MIN_COLUMN_PORTION),
            usize::from(TABLE_MAX_COLUMN_PORTION),
        )
        .try_into()
        .expect("table column portion is clamped to u16")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HighlightSegment<'a> {
    text: &'a str,
    highlight: SearchHighlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchHighlight {
    None,
    Match,
    Active,
}

#[derive(Debug, Clone, Copy)]
struct SearchContext<'a> {
    query: &'a str,
    active_line: Option<&'a str>,
}

impl<'a> SearchContext<'a> {
    fn new(query: Option<&'a str>, active_line: Option<&'a str>) -> Option<Self> {
        normalized_search_query(query).map(|query| Self { query, active_line })
    }
}

#[derive(Debug, Clone, Copy)]
struct RenderSearch<'a> {
    query: &'a str,
    active: bool,
}

fn normalized_search_query(query: Option<&str>) -> Option<&str> {
    query.and_then(|query| {
        let trimmed = query.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn render_search_context<'a>(
    block: &Block,
    context: Option<SearchContext<'a>>,
) -> Option<RenderSearch<'a>> {
    context.map(|context| RenderSearch {
        query: context.query,
        active: context.active_line.is_some_and(|active_line| {
            block_matches_active_search_line(block, active_line, context.query)
        }),
    })
}

fn block_matches_active_search_line(block: &Block, active_line: &str, query: &str) -> bool {
    let active_line = searchable_text(active_line);
    let block_text = searchable_text(&block_plain_text(block));
    let query = searchable_text(query);

    !query.is_empty()
        && active_line.contains(&query)
        && block_text.contains(&query)
        && (active_line.contains(block_text.trim()) || block_text.contains(active_line.trim()))
}

fn block_plain_text(block: &Block) -> String {
    match block {
        Block::Heading { spans, .. } | Block::Paragraph(spans) | Block::BlockQuote(spans) => {
            inline::plain_text(spans)
        }
        Block::List { items, .. } => items
            .iter()
            .map(|item| inline::plain_text(&item.content))
            .collect::<Vec<_>>()
            .join(" "),
        Block::Table { header, rows, .. } => header
            .iter()
            .chain(rows.iter().flat_map(|row| row.iter()))
            .map(|cell| inline::plain_text(cell))
            .collect::<Vec<_>>()
            .join(" "),
        Block::CodeBlock { code, .. } | Block::Diagram { source: code, .. } => code.clone(),
        Block::Image { alt, url, title } => [alt.as_str(), url.as_str(), title.as_str()].join(" "),
        Block::Math { source, .. } => source.clone(),
        Block::Rule => String::new(),
    }
}

fn searchable_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn highlight_segments<'a>(
    text: &'a str,
    search: Option<RenderSearch<'_>>,
) -> Vec<HighlightSegment<'a>> {
    let Some(search) = search else {
        return vec![HighlightSegment {
            text,
            highlight: SearchHighlight::None,
        }];
    };
    let lower_text = text.to_lowercase();
    let lower_query = search.query.to_lowercase();
    let highlight = if search.active {
        SearchHighlight::Active
    } else {
        SearchHighlight::Match
    };
    let mut segments = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = lower_text[cursor..].find(&lower_query) {
        let start = cursor + relative_start;
        let end = start + lower_query.len();

        if start > cursor {
            segments.push(HighlightSegment {
                text: &text[cursor..start],
                highlight: SearchHighlight::None,
            });
        }

        segments.push(HighlightSegment {
            text: &text[start..end],
            highlight,
        });
        cursor = end;
    }

    if cursor < text.len() {
        segments.push(HighlightSegment {
            text: &text[cursor..],
            highlight: SearchHighlight::None,
        });
    }

    if segments.is_empty() {
        segments.push(HighlightSegment {
            text,
            highlight: SearchHighlight::None,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use paperview_core::parser::parse_markdown;

    use super::{
        HighlightSegment, RenderSearch, SearchHighlight, active_heading_for_scroll,
        block_matches_active_search_line, heading_scroll_progress, highlight_segments,
        image_dimensions_from_bytes, image_dimensions_from_path, is_fetchable_remote_image_url,
        resolve_image_path, table_cell_portion, table_column_portions,
    };

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
    fn splits_text_into_search_highlight_segments() {
        assert_eq!(
            highlight_segments(
                "PaperView paper reader",
                Some(RenderSearch {
                    query: "paper",
                    active: false
                })
            ),
            vec![
                HighlightSegment {
                    text: "Paper",
                    highlight: SearchHighlight::Match
                },
                HighlightSegment {
                    text: "View ",
                    highlight: SearchHighlight::None
                },
                HighlightSegment {
                    text: "paper",
                    highlight: SearchHighlight::Match
                },
                HighlightSegment {
                    text: " reader",
                    highlight: SearchHighlight::None
                }
            ]
        );
    }

    #[test]
    fn marks_active_search_highlight_segments() {
        assert_eq!(
            highlight_segments(
                "PaperView paper reader",
                Some(RenderSearch {
                    query: "paper",
                    active: true
                })
            ),
            vec![
                HighlightSegment {
                    text: "Paper",
                    highlight: SearchHighlight::Active
                },
                HighlightSegment {
                    text: "View ",
                    highlight: SearchHighlight::None
                },
                HighlightSegment {
                    text: "paper",
                    highlight: SearchHighlight::Active
                },
                HighlightSegment {
                    text: " reader",
                    highlight: SearchHighlight::None
                }
            ]
        );
    }

    #[test]
    fn ignores_empty_search_highlight_queries() {
        assert_eq!(
            highlight_segments("PaperView", None),
            vec![HighlightSegment {
                text: "PaperView",
                highlight: SearchHighlight::None
            }]
        );
    }

    #[test]
    fn matches_active_search_line_to_rendered_block_text() {
        let parsed = parse_markdown("# Title\n\nA **needle** here.\n\nAnother needle.");

        assert!(block_matches_active_search_line(
            &parsed.blocks[1],
            "A **needle** here.",
            "needle"
        ));
        assert!(!block_matches_active_search_line(
            &parsed.blocks[2],
            "A **needle** here.",
            "needle"
        ));
    }

    #[test]
    fn responsive_table_columns_use_shared_bounded_widths() {
        let parsed = parse_markdown(
            "| Short | Description |\n| --- | --- |\n| A | Supercalifragilisticexpialidocious detail |\n| Longer label | ok |",
        );
        let paperview_core::parser::Block::Table { header, rows, .. } = &parsed.blocks[0] else {
            panic!("expected table block");
        };

        assert_eq!(table_column_portions(header, rows), vec![12, 32]);
    }

    #[test]
    fn responsive_table_columns_keep_empty_cells_readable() {
        let parsed = parse_markdown("| A | B |\n| --- | --- |\n|  | ok |");
        let paperview_core::parser::Block::Table { header, rows, .. } = &parsed.blocks[0] else {
            panic!("expected table block");
        };

        assert_eq!(table_cell_portion(&rows[0][0]), 8);
        assert_eq!(table_column_portions(header, rows), vec![8, 8]);
    }

    #[test]
    fn remote_image_urls_do_not_resolve_to_local_previews() {
        assert_eq!(
            resolve_image_path("https://example.com/image.png", None),
            None
        );
        assert_eq!(resolve_image_path("data:image/png;base64,AAAA", None), None);
    }

    #[test]
    fn only_http_urls_are_fetchable_remote_images() {
        assert!(is_fetchable_remote_image_url(
            "https://example.com/image.png"
        ));
        assert!(is_fetchable_remote_image_url(
            "http://example.com/image.png"
        ));
        assert!(!is_fetchable_remote_image_url("data:image/png;base64,AAAA"));
        assert!(!is_fetchable_remote_image_url("file:///tmp/image.png"));
        assert!(!is_fetchable_remote_image_url("assets/image.png"));
    }

    #[test]
    fn reads_png_dimensions_from_bytes_and_path() {
        let png = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 13, b'I', b'H', b'D', b'R',
            0, 0, 1, 0x40, 0, 0, 0, 0xf0, 8, 2, 0, 0, 0,
        ];
        let dir = temp_dir("image-dimensions");
        let path = dir.join("preview.png");

        fs::create_dir_all(&dir).expect("create temp dir");
        fs::write(&path, png).expect("write png header");

        let dimensions = image_dimensions_from_bytes(&png).expect("png dimensions");
        assert_eq!(dimensions.width, 320);
        assert_eq!(dimensions.height, 240);
        assert_eq!(dimensions.label(), "320 x 240 px");
        assert_eq!(image_dimensions_from_path(&path), Some(dimensions));

        fs::remove_dir_all(dir).expect("remove temp dir");
    }

    #[test]
    fn reads_gif_jpeg_and_webp_dimensions_from_bytes() {
        let gif = [b'G', b'I', b'F', b'8', b'9', b'a', 0x40, 0x01, 0xf0, 0x00];
        let jpeg = [
            0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x00,
            0xf0, 0x01, 0x40, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00,
        ];
        let webp = [
            b'R', b'I', b'F', b'F', 22, 0, 0, 0, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b'X',
            10, 0, 0, 0, 0, 0, 0, 0, 0x3f, 0x01, 0, 0xef, 0, 0,
        ];

        assert_eq!(
            image_dimensions_from_bytes(&gif),
            Some(super::ImageDimensions {
                width: 320,
                height: 240
            })
        );
        assert_eq!(
            image_dimensions_from_bytes(&jpeg),
            Some(super::ImageDimensions {
                width: 320,
                height: 240
            })
        );
        assert_eq!(
            image_dimensions_from_bytes(&webp),
            Some(super::ImageDimensions {
                width: 320,
                height: 240
            })
        );
        assert_eq!(image_dimensions_from_bytes(b"not an image"), None);
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("paperview-reader-{nanos}-{name}"))
    }
}
