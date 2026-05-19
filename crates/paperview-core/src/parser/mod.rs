pub mod elements;

use std::collections::HashMap;

use pulldown_cmark::{
    Alignment as CmarkAlignment, CodeBlockKind, Event, HeadingLevel as CmarkHeadingLevel, Options,
    Parser, Tag, TagEnd,
};

use self::elements::{diagram, image, inline, math, table};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDocument {
    pub blocks: Vec<Block>,
}

impl ParsedDocument {
    #[must_use]
    pub fn title(&self) -> Option<String> {
        self.blocks.iter().find_map(|block| match block {
            Block::Heading {
                level: HeadingLevel::H1,
                spans,
            } => Some(heading_text(spans)),
            _ => None,
        })
    }

    #[must_use]
    pub fn toc(&self) -> Vec<TocItem> {
        let mut seen_slugs = HashMap::<String, usize>::new();

        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(block_index, block)| match block {
                Block::Heading { level, spans } => {
                    let title = heading_text(spans);
                    let base_slug = slugify(&title);
                    let count = seen_slugs.entry(base_slug.clone()).or_insert(0);
                    *count += 1;

                    let slug = if *count == 1 {
                        base_slug
                    } else {
                        format!("{base_slug}-{count}")
                    };

                    Some(TocItem {
                        level: *level,
                        title,
                        slug,
                        block_index,
                    })
                }
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocItem {
    pub level: HeadingLevel,
    pub title: String,
    pub slug: String,
    pub block_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: HeadingLevel,
        spans: Vec<InlineSpan>,
    },
    Paragraph(Vec<InlineSpan>),
    BlockQuote(Vec<InlineSpan>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Diagram {
        language: String,
        source: String,
    },
    Image {
        alt: String,
        url: String,
        title: String,
    },
    Table {
        alignments: Vec<TableAlignment>,
        header: TableRow,
        rows: Vec<TableRow>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Math {
        display: bool,
        source: String,
    },
    Rule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineSpan {
    pub text: String,
    pub strong: bool,
    pub emphasis: bool,
    pub code: bool,
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub source_line: Option<usize>,
    pub content: Vec<InlineSpan>,
}

pub type TableCell = Vec<InlineSpan>;
pub type TableRow = Vec<TableCell>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

impl From<CmarkAlignment> for TableAlignment {
    fn from(alignment: CmarkAlignment) -> Self {
        match alignment {
            CmarkAlignment::None => Self::None,
            CmarkAlignment::Left => Self::Left,
            CmarkAlignment::Center => Self::Center,
            CmarkAlignment::Right => Self::Right,
        }
    }
}

impl HeadingLevel {
    #[must_use]
    pub fn as_depth(self) -> u8 {
        match self {
            Self::H1 => 1,
            Self::H2 => 2,
            Self::H3 => 3,
            Self::H4 => 4,
            Self::H5 => 5,
            Self::H6 => 6,
        }
    }
}

impl From<CmarkHeadingLevel> for HeadingLevel {
    fn from(level: CmarkHeadingLevel) -> Self {
        match level {
            CmarkHeadingLevel::H1 => Self::H1,
            CmarkHeadingLevel::H2 => Self::H2,
            CmarkHeadingLevel::H3 => Self::H3,
            CmarkHeadingLevel::H4 => Self::H4,
            CmarkHeadingLevel::H5 => Self::H5,
            CmarkHeadingLevel::H6 => Self::H6,
        }
    }
}

#[must_use]
pub fn parse_markdown(source: &str) -> ParsedDocument {
    let mut builder = DocumentBuilder::default();

    for event in Parser::new_ext(source, Options::all()) {
        builder.push(event);
    }

    let mut blocks = builder.finish();
    assign_task_source_lines(&mut blocks, source);

    ParsedDocument { blocks }
}

#[derive(Debug)]
enum OpenBlock {
    Heading {
        level: HeadingLevel,
        spans: Vec<InlineSpan>,
    },
    Paragraph(Vec<InlineSpan>),
    BlockQuote(Vec<InlineSpan>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
}

#[derive(Debug, Default)]
struct OpenList {
    ordered: bool,
    items: Vec<ListItem>,
    current_item: Option<ListItem>,
}

#[derive(Debug)]
struct OpenTable {
    alignments: Vec<TableAlignment>,
    header: TableRow,
    rows: Vec<TableRow>,
    current_row: Option<TableRow>,
    current_cell: Option<TableCell>,
    in_header: bool,
}

#[derive(Debug)]
struct OpenImage {
    alt: String,
    url: String,
    title: String,
}

#[derive(Debug, Default)]
struct DocumentBuilder {
    blocks: Vec<Block>,
    open_block: Option<OpenBlock>,
    open_list: Option<OpenList>,
    open_table: Option<OpenTable>,
    open_image: Option<OpenImage>,
    inline_state: inline::InlineState,
}

impl DocumentBuilder {
    fn push(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.push_text(&text),
            Event::Code(code) => self.push_code(&code),
            Event::InlineMath(source) => self.push_text(&math::inline_text(&source)),
            Event::DisplayMath(source) => self.push_display_math(&source),
            Event::TaskListMarker(checked) => self.push_task_list_marker(checked),
            Event::SoftBreak | Event::HardBreak => self.push_text("\n"),
            Event::Rule => self.blocks.push(Block::Rule),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.open_block = Some(OpenBlock::Heading {
                    level: level.into(),
                    spans: Vec::new(),
                });
            }
            Tag::Paragraph if self.open_list.is_none() && self.open_block.is_none() => {
                self.open_block = Some(OpenBlock::Paragraph(Vec::new()));
            }
            Tag::Paragraph => {}
            Tag::Strong => self.inline_state.strong_depth += 1,
            Tag::Emphasis => self.inline_state.emphasis_depth += 1,
            Tag::Link { dest_url, .. } => {
                self.inline_state.links.push(dest_url.into_string());
            }
            Tag::BlockQuote(_) => {
                self.open_block = Some(OpenBlock::BlockQuote(Vec::new()));
            }
            Tag::CodeBlock(kind) => {
                self.open_block = Some(OpenBlock::CodeBlock {
                    language: code_block_language(kind),
                    code: String::new(),
                });
            }
            Tag::Table(alignments) => {
                self.close_block();
                self.open_table = Some(OpenTable {
                    alignments: table::alignments(alignments),
                    header: Vec::new(),
                    rows: Vec::new(),
                    current_row: None,
                    current_cell: None,
                    in_header: false,
                });
            }
            Tag::TableHead => {
                if let Some(table) = &mut self.open_table {
                    table.in_header = true;
                    table.current_row = Some(Vec::new());
                }
            }
            Tag::TableRow => {
                if let Some(table) = &mut self.open_table {
                    table.current_row = Some(Vec::new());
                }
            }
            Tag::TableCell => {
                if let Some(table) = &mut self.open_table {
                    table.current_cell = Some(Vec::new());
                }
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                self.open_image = Some(OpenImage {
                    alt: String::new(),
                    url: dest_url.into_string(),
                    title: title.into_string(),
                });
            }
            Tag::List(first_item) => {
                self.open_list = Some(OpenList {
                    ordered: first_item.is_some(),
                    items: Vec::new(),
                    current_item: None,
                });
            }
            Tag::Item => {
                if let Some(list) = &mut self.open_list {
                    list.current_item = Some(ListItem {
                        checked: None,
                        source_line: None,
                        content: Vec::new(),
                    });
                }
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => self.close_block(),
            TagEnd::Paragraph if matches!(self.open_block, Some(OpenBlock::Paragraph(_))) => {
                self.close_block();
            }
            TagEnd::Paragraph => {}
            TagEnd::Strong => {
                self.inline_state.strong_depth = self.inline_state.strong_depth.saturating_sub(1);
            }
            TagEnd::Emphasis => {
                self.inline_state.emphasis_depth =
                    self.inline_state.emphasis_depth.saturating_sub(1);
            }
            TagEnd::Link => {
                self.inline_state.links.pop();
            }
            TagEnd::BlockQuote(_) => self.close_block(),
            TagEnd::CodeBlock => self.close_block(),
            TagEnd::TableCell => {
                if let Some(table) = &mut self.open_table
                    && let Some(cell) = table.current_cell.take()
                    && let Some(row) = &mut table.current_row
                {
                    row.push(cell);
                }
            }
            TagEnd::TableRow => {
                if let Some(table) = &mut self.open_table
                    && let Some(row) = table.current_row.take()
                {
                    if table.in_header {
                        table.header = row;
                    } else {
                        table.rows.push(row);
                    }
                }
            }
            TagEnd::TableHead => {
                if let Some(table) = &mut self.open_table {
                    if let Some(row) = table.current_row.take() {
                        table.header = row;
                    }
                    table.in_header = false;
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.open_table.take() {
                    self.blocks.push(Block::Table {
                        alignments: table.alignments,
                        header: table.header,
                        rows: table.rows,
                    });
                }
            }
            TagEnd::Image => self.close_image(),
            TagEnd::Item => {
                if let Some(list) = &mut self.open_list
                    && let Some(item) = list.current_item.take()
                {
                    list.items.push(item);
                }
            }
            TagEnd::List(_) => {
                if let Some(list) = self.open_list.take() {
                    self.blocks.push(Block::List {
                        ordered: list.ordered,
                        items: list.items,
                    });
                }
            }
            _ => {}
        }
    }

    fn push_text(&mut self, text: &str) {
        if let Some(image) = &mut self.open_image {
            image.alt.push_str(text);
            return;
        }

        if let Some(table) = &mut self.open_table
            && let Some(cell) = &mut table.current_cell
        {
            inline::push_span(cell, inline::span(text, &self.inline_state));
            return;
        }

        if let Some(list) = &mut self.open_list
            && let Some(item) = &mut list.current_item
        {
            inline::push_span(&mut item.content, inline::span(text, &self.inline_state));
            return;
        }

        match &mut self.open_block {
            Some(OpenBlock::Heading { spans, .. }) => {
                inline::push_span(spans, inline::span(text, &self.inline_state));
            }
            Some(OpenBlock::BlockQuote(spans)) => {
                inline::push_span(spans, inline::span(text, &self.inline_state));
            }
            Some(OpenBlock::Paragraph(spans)) => {
                inline::push_span(spans, inline::span(text, &self.inline_state));
            }
            Some(OpenBlock::CodeBlock { code, .. }) => code.push_str(text),
            None => {}
        }
    }

    fn push_code(&mut self, code: &str) {
        if let Some(table) = &mut self.open_table
            && let Some(cell) = &mut table.current_cell
        {
            inline::push_span(cell, inline::code_span(code, &self.inline_state));
            return;
        }

        if let Some(list) = &mut self.open_list
            && let Some(item) = &mut list.current_item
        {
            inline::push_span(
                &mut item.content,
                inline::code_span(code, &self.inline_state),
            );
            return;
        }

        match &mut self.open_block {
            Some(OpenBlock::Heading { spans, .. })
            | Some(OpenBlock::Paragraph(spans))
            | Some(OpenBlock::BlockQuote(spans)) => {
                inline::push_span(spans, inline::code_span(code, &self.inline_state));
            }
            _ => {
                self.push_text("`");
                self.push_text(code);
                self.push_text("`");
            }
        }
    }

    fn close_image(&mut self) {
        let Some(open_image) = self.open_image.take() else {
            return;
        };
        let alt = image::alt_text(&open_image.alt);
        let markdown = image::markdown_text(&alt, &open_image.url, &open_image.title);

        if self.open_table.is_some() || self.open_list.is_some() {
            self.push_text(&markdown);
            return;
        }

        let is_standalone =
            matches!(&self.open_block, Some(OpenBlock::Paragraph(spans)) if spans.is_empty());
        if is_standalone {
            self.close_block();
            self.blocks.push(Block::Image {
                alt,
                url: open_image.url,
                title: open_image.title,
            });
        } else {
            self.push_text(&markdown);
        }
    }

    fn push_display_math(&mut self, source: &str) {
        self.close_block();
        self.blocks.push(Block::Math {
            display: true,
            source: math::display_source(source),
        });
    }

    fn push_task_list_marker(&mut self, checked: bool) {
        if let Some(list) = &mut self.open_list
            && let Some(item) = &mut list.current_item
        {
            item.checked = Some(checked);
        }
    }

    fn close_block(&mut self) {
        let Some(open_block) = self.open_block.take() else {
            return;
        };

        match open_block {
            OpenBlock::Heading { level, spans } => {
                if !inline::plain_text(&spans).trim().is_empty() {
                    self.blocks.push(Block::Heading { level, spans });
                }
            }
            OpenBlock::Paragraph(spans) => {
                if !inline::plain_text(&spans).trim().is_empty() {
                    self.blocks.push(Block::Paragraph(spans));
                }
            }
            OpenBlock::BlockQuote(spans) => {
                self.blocks.push(Block::BlockQuote(spans));
            }
            OpenBlock::CodeBlock { language, code } => {
                if diagram::is_mermaid(language.as_deref()) {
                    self.blocks.push(Block::Diagram {
                        language: diagram::MERMAID_LANGUAGE.to_owned(),
                        source: diagram::source(&code),
                    });
                } else {
                    self.blocks.push(Block::CodeBlock { language, code });
                }
            }
        }
    }

    fn finish(self) -> Vec<Block> {
        self.blocks
    }
}

fn code_block_language(kind: CodeBlockKind<'_>) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(language) if !language.is_empty() => Some(language.into_string()),
        _ => None,
    }
}

fn heading_text(spans: &[InlineSpan]) -> String {
    normalize_text(&inline::plain_text(spans))
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in text.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('-');
            previous_was_separator = true;
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "section".to_owned()
    } else {
        slug
    }
}

fn assign_task_source_lines(blocks: &mut [Block], source: &str) {
    let mut task_lines = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| task_marker_range(line).map(|_| index));

    for block in blocks {
        if let Block::List { items, .. } = block {
            for item in items {
                if item.checked.is_some() {
                    item.source_line = task_lines.next();
                }
            }
        }
    }
}

pub(crate) fn task_marker_range(line: &str) -> Option<(usize, bool)> {
    let indent_len = line.len() - line.trim_start().len();
    let bytes = line.as_bytes();
    let mut index = indent_len;

    match bytes.get(index).copied()? {
        b'-' | b'+' | b'*' => {
            index += 1;
        }
        b'0'..=b'9' => {
            while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                index += 1;
            }
            if !matches!(bytes.get(index), Some(b'.' | b')')) {
                return None;
            }
            index += 1;
        }
        _ => return None,
    }

    if !matches!(bytes.get(index), Some(b' ' | b'\t')) {
        return None;
    }
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }

    let marker = bytes.get(index..index + 3)?;
    match marker {
        b"[ ]" => Some((index + 1, false)),
        b"[x]" | b"[X]" => Some((index + 1, true)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Block, HeadingLevel, InlineSpan, ListItem, TableAlignment, TocItem, parse_markdown,
    };

    fn text_span(text: &str) -> InlineSpan {
        InlineSpan {
            text: text.to_owned(),
            strong: false,
            emphasis: false,
            code: false,
            link: None,
        }
    }

    fn list_item(content: Vec<InlineSpan>) -> ListItem {
        ListItem {
            checked: None,
            source_line: None,
            content,
        }
    }

    fn task_item_at_line(checked: bool, source_line: usize, content: Vec<InlineSpan>) -> ListItem {
        ListItem {
            checked: Some(checked),
            source_line: Some(source_line),
            content,
        }
    }

    #[test]
    fn parses_headings_and_paragraphs() {
        let parsed = parse_markdown("# PaperView\n\nNative Markdown viewer.");

        assert_eq!(
            parsed.blocks,
            vec![
                Block::Heading {
                    level: HeadingLevel::H1,
                    spans: vec![text_span("PaperView")]
                },
                Block::Paragraph(vec![text_span("Native Markdown viewer.")])
            ]
        );
    }

    #[test]
    fn exposes_first_h1_as_title() {
        let parsed = parse_markdown("## Preface\n\n# PaperView");

        assert_eq!(parsed.title(), Some("PaperView".to_owned()));
    }

    #[test]
    fn preserves_heading_inline_spans() {
        let parsed = parse_markdown("# **PaperView** [docs](docs/index.md) `reader`");

        assert_eq!(
            parsed.blocks,
            vec![Block::Heading {
                level: HeadingLevel::H1,
                spans: vec![
                    InlineSpan {
                        text: "PaperView".to_owned(),
                        strong: true,
                        emphasis: false,
                        code: false,
                        link: None
                    },
                    text_span(" "),
                    InlineSpan {
                        text: "docs".to_owned(),
                        strong: false,
                        emphasis: false,
                        code: false,
                        link: Some("docs/index.md".to_owned())
                    },
                    text_span(" "),
                    InlineSpan {
                        text: "reader".to_owned(),
                        strong: false,
                        emphasis: false,
                        code: true,
                        link: None
                    }
                ]
            }]
        );

        assert_eq!(parsed.title(), Some("PaperView docs reader".to_owned()));
    }

    #[test]
    fn parses_common_block_elements() {
        let parsed = parse_markdown(
            "> Quiet reader\n\n- Fast\n- Native\n\n```rust\nfn main() {}\n```\n\n---",
        );

        assert_eq!(
            parsed.blocks,
            vec![
                Block::BlockQuote(vec![text_span("Quiet reader")]),
                Block::List {
                    ordered: false,
                    items: vec![
                        list_item(vec![text_span("Fast")]),
                        list_item(vec![text_span("Native")])
                    ]
                },
                Block::CodeBlock {
                    language: Some("rust".to_owned()),
                    code: "fn main() {}\n".to_owned()
                },
                Block::Rule
            ]
        );
    }

    #[test]
    fn preserves_latex_math_source() {
        let parsed = parse_markdown("Before $x + y$.\n\n$$\nE = mc^2\n$$\n\nAfter.");

        assert_eq!(
            parsed.blocks,
            vec![
                Block::Paragraph(vec![text_span("Before $x + y$.")]),
                Block::Math {
                    display: true,
                    source: "E = mc^2".to_owned()
                },
                Block::Paragraph(vec![text_span("After.")])
            ]
        );
    }

    #[test]
    fn preserves_paragraph_inline_spans() {
        let parsed =
            parse_markdown("A **bold** and *quiet* [link](https://example.com) with `code`.");

        assert_eq!(
            parsed.blocks,
            vec![Block::Paragraph(vec![
                text_span("A "),
                InlineSpan {
                    text: "bold".to_owned(),
                    strong: true,
                    emphasis: false,
                    code: false,
                    link: None
                },
                text_span(" and "),
                InlineSpan {
                    text: "quiet".to_owned(),
                    strong: false,
                    emphasis: true,
                    code: false,
                    link: None
                },
                text_span(" "),
                InlineSpan {
                    text: "link".to_owned(),
                    strong: false,
                    emphasis: false,
                    code: false,
                    link: Some("https://example.com".to_owned())
                },
                text_span(" with "),
                InlineSpan {
                    text: "code".to_owned(),
                    strong: false,
                    emphasis: false,
                    code: true,
                    link: None
                },
                text_span(".")
            ])]
        );
    }

    #[test]
    fn preserves_list_and_blockquote_inline_spans() {
        let parsed = parse_markdown(
            "> A **quiet** [quote](https://example.com)\n\n- *Fast* `reader`\n- Plain",
        );

        assert_eq!(
            parsed.blocks,
            vec![
                Block::BlockQuote(vec![
                    text_span("A "),
                    InlineSpan {
                        text: "quiet".to_owned(),
                        strong: true,
                        emphasis: false,
                        code: false,
                        link: None
                    },
                    text_span(" "),
                    InlineSpan {
                        text: "quote".to_owned(),
                        strong: false,
                        emphasis: false,
                        code: false,
                        link: Some("https://example.com".to_owned())
                    }
                ]),
                Block::List {
                    ordered: false,
                    items: vec![
                        list_item(vec![
                            InlineSpan {
                                text: "Fast".to_owned(),
                                strong: false,
                                emphasis: true,
                                code: false,
                                link: None
                            },
                            text_span(" "),
                            InlineSpan {
                                text: "reader".to_owned(),
                                strong: false,
                                emphasis: false,
                                code: true,
                                link: None
                            }
                        ]),
                        list_item(vec![text_span("Plain")])
                    ]
                }
            ]
        );
    }

    #[test]
    fn preserves_task_list_markers() {
        let parsed = parse_markdown("- [x] Done\n- [ ] Todo\n- Plain");

        assert_eq!(
            parsed.blocks,
            vec![Block::List {
                ordered: false,
                items: vec![
                    task_item_at_line(true, 0, vec![text_span("Done")]),
                    task_item_at_line(false, 1, vec![text_span("Todo")]),
                    list_item(vec![text_span("Plain")])
                ]
            }]
        );
    }

    #[test]
    fn preserves_task_list_source_lines() {
        let parsed = parse_markdown("Intro\n\n- [ ] First\n- [x] Second\n\n1. [X] Ordered");

        assert_eq!(
            parsed.blocks,
            vec![
                Block::Paragraph(vec![text_span("Intro")]),
                Block::List {
                    ordered: false,
                    items: vec![
                        task_item_at_line(false, 2, vec![text_span("First")]),
                        task_item_at_line(true, 3, vec![text_span("Second")])
                    ]
                },
                Block::List {
                    ordered: true,
                    items: vec![task_item_at_line(true, 5, vec![text_span("Ordered")])]
                }
            ]
        );
    }

    #[test]
    fn parses_mermaid_fences_as_diagram_blocks() {
        let parsed =
            parse_markdown("```mermaid\ngraph TD\n  A-->B\n```\n\n```rust\nfn main() {}\n```");

        assert_eq!(
            parsed.blocks,
            vec![
                Block::Diagram {
                    language: "mermaid".to_owned(),
                    source: "graph TD\n  A-->B".to_owned()
                },
                Block::CodeBlock {
                    language: Some("rust".to_owned()),
                    code: "fn main() {}\n".to_owned()
                }
            ]
        );
    }

    #[test]
    fn parses_markdown_tables() {
        let parsed = parse_markdown(
            "| Feature | Status |\n| :--- | ---: |\n| GUI | Done |\n| TUI | In progress |",
        );

        assert_eq!(
            parsed.blocks,
            vec![Block::Table {
                alignments: vec![TableAlignment::Left, TableAlignment::Right],
                header: vec![vec![text_span("Feature")], vec![text_span("Status")]],
                rows: vec![
                    vec![vec![text_span("GUI")], vec![text_span("Done")]],
                    vec![vec![text_span("TUI")], vec![text_span("In progress")]]
                ]
            }]
        );
    }

    #[test]
    fn preserves_table_cell_inline_spans() {
        let parsed =
            parse_markdown("| Feature | Link |\n| --- | --- |\n| **GUI** | [docs](docs/gui.md) |");

        assert_eq!(
            parsed.blocks,
            vec![Block::Table {
                alignments: vec![TableAlignment::None, TableAlignment::None],
                header: vec![vec![text_span("Feature")], vec![text_span("Link")]],
                rows: vec![vec![
                    vec![InlineSpan {
                        text: "GUI".to_owned(),
                        strong: true,
                        emphasis: false,
                        code: false,
                        link: None
                    }],
                    vec![InlineSpan {
                        text: "docs".to_owned(),
                        strong: false,
                        emphasis: false,
                        code: false,
                        link: Some("docs/gui.md".to_owned())
                    }]
                ]]
            }]
        );
    }

    #[test]
    fn parses_standalone_images_as_blocks() {
        let parsed = parse_markdown("![Architecture diagram](docs/arch.png \"Architecture\")");

        assert_eq!(
            parsed.blocks,
            vec![Block::Image {
                alt: "Architecture diagram".to_owned(),
                url: "docs/arch.png".to_owned(),
                title: "Architecture".to_owned()
            }]
        );
    }

    #[test]
    fn preserves_inline_images_inside_paragraphs() {
        let parsed = parse_markdown("See ![Architecture](docs/arch.png) for details.");

        assert_eq!(
            parsed.blocks,
            vec![Block::Paragraph(vec![text_span(
                "See ![Architecture](docs/arch.png) for details."
            )])]
        );
    }

    #[test]
    fn derives_toc_from_heading_blocks() {
        let parsed = parse_markdown("# Intro\n\nText.\n\n## Details\n\n### Details!");

        assert_eq!(
            parsed.toc(),
            vec![
                TocItem {
                    level: HeadingLevel::H1,
                    title: "Intro".to_owned(),
                    slug: "intro".to_owned(),
                    block_index: 0
                },
                TocItem {
                    level: HeadingLevel::H2,
                    title: "Details".to_owned(),
                    slug: "details".to_owned(),
                    block_index: 2
                },
                TocItem {
                    level: HeadingLevel::H3,
                    title: "Details!".to_owned(),
                    slug: "details-2".to_owned(),
                    block_index: 3
                }
            ]
        );
    }
}
