pub mod elements;

use std::collections::HashMap;

use pulldown_cmark::{
    Alignment as CmarkAlignment, CodeBlockKind, Event, HeadingLevel as CmarkHeadingLevel, Options,
    Parser, Tag, TagEnd,
};

use self::elements::{diagram, image, math, table};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDocument {
    pub blocks: Vec<Block>,
}

impl ParsedDocument {
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.blocks.iter().find_map(|block| match block {
            Block::Heading {
                level: HeadingLevel::H1,
                text,
            } => Some(text.as_str()),
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
                Block::Heading { level, text } => {
                    let base_slug = slugify(text);
                    let count = seen_slugs.entry(base_slug.clone()).or_insert(0);
                    *count += 1;

                    let slug = if *count == 1 {
                        base_slug
                    } else {
                        format!("{base_slug}-{count}")
                    };

                    Some(TocItem {
                        level: *level,
                        title: text.clone(),
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
        text: String,
    },
    Paragraph(String),
    BlockQuote(String),
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
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    List {
        ordered: bool,
        items: Vec<String>,
    },
    Math {
        display: bool,
        source: String,
    },
    Rule,
}

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

    ParsedDocument {
        blocks: builder.finish(),
    }
}

#[derive(Debug)]
enum OpenBlock {
    Heading {
        level: HeadingLevel,
        text: String,
    },
    Paragraph(String),
    BlockQuote(String),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
}

#[derive(Debug, Default)]
struct OpenList {
    ordered: bool,
    items: Vec<String>,
    current_item: Option<String>,
}

#[derive(Debug)]
struct OpenTable {
    alignments: Vec<TableAlignment>,
    header: Vec<String>,
    rows: Vec<Vec<String>>,
    current_row: Option<Vec<String>>,
    current_cell: Option<String>,
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
}

impl DocumentBuilder {
    fn push(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.push_text(&text),
            Event::Code(code) => {
                self.push_text("`");
                self.push_text(&code);
                self.push_text("`");
            }
            Event::InlineMath(source) => self.push_text(&math::inline_text(&source)),
            Event::DisplayMath(source) => self.push_display_math(&source),
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
                    text: String::new(),
                });
            }
            Tag::Paragraph if self.open_list.is_none() && self.open_block.is_none() => {
                self.open_block = Some(OpenBlock::Paragraph(String::new()));
            }
            Tag::Paragraph => {}
            Tag::BlockQuote(_) => {
                self.open_block = Some(OpenBlock::BlockQuote(String::new()));
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
                    table.current_cell = Some(String::new());
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
                    list.current_item = Some(String::new());
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
            TagEnd::BlockQuote(_) => self.close_block(),
            TagEnd::CodeBlock => self.close_block(),
            TagEnd::TableCell => {
                if let Some(table) = &mut self.open_table
                    && let Some(cell) = table.current_cell.take()
                    && let Some(row) = &mut table.current_row
                {
                    row.push(table::cell_text(&cell));
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
                    list.items.push(normalize_text(&item));
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
            cell.push_str(text);
            return;
        }

        if let Some(list) = &mut self.open_list
            && let Some(item) = &mut list.current_item
        {
            item.push_str(text);
            return;
        }

        match &mut self.open_block {
            Some(OpenBlock::Heading { text: target, .. })
            | Some(OpenBlock::Paragraph(target))
            | Some(OpenBlock::BlockQuote(target)) => target.push_str(text),
            Some(OpenBlock::CodeBlock { code, .. }) => code.push_str(text),
            None => {}
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
            matches!(&self.open_block, Some(OpenBlock::Paragraph(text)) if text.is_empty());
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

    fn close_block(&mut self) {
        let Some(open_block) = self.open_block.take() else {
            return;
        };

        match open_block {
            OpenBlock::Heading { level, text } => self.blocks.push(Block::Heading {
                level,
                text: normalize_text(&text),
            }),
            OpenBlock::Paragraph(text) => {
                let text = normalize_text(&text);
                if !text.is_empty() {
                    self.blocks.push(Block::Paragraph(text));
                }
            }
            OpenBlock::BlockQuote(text) => {
                self.blocks.push(Block::BlockQuote(normalize_text(&text)));
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

#[cfg(test)]
mod tests {
    use super::{Block, HeadingLevel, TableAlignment, TocItem, parse_markdown};

    #[test]
    fn parses_headings_and_paragraphs() {
        let parsed = parse_markdown("# PaperView\n\nNative Markdown viewer.");

        assert_eq!(
            parsed.blocks,
            vec![
                Block::Heading {
                    level: HeadingLevel::H1,
                    text: "PaperView".to_owned()
                },
                Block::Paragraph("Native Markdown viewer.".to_owned())
            ]
        );
    }

    #[test]
    fn exposes_first_h1_as_title() {
        let parsed = parse_markdown("## Preface\n\n# PaperView");

        assert_eq!(parsed.title(), Some("PaperView"));
    }

    #[test]
    fn parses_common_block_elements() {
        let parsed = parse_markdown(
            "> Quiet reader\n\n- Fast\n- Native\n\n```rust\nfn main() {}\n```\n\n---",
        );

        assert_eq!(
            parsed.blocks,
            vec![
                Block::BlockQuote("Quiet reader".to_owned()),
                Block::List {
                    ordered: false,
                    items: vec!["Fast".to_owned(), "Native".to_owned()]
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
                Block::Paragraph("Before $x + y$.".to_owned()),
                Block::Math {
                    display: true,
                    source: "E = mc^2".to_owned()
                },
                Block::Paragraph("After.".to_owned())
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
                header: vec!["Feature".to_owned(), "Status".to_owned()],
                rows: vec![
                    vec!["GUI".to_owned(), "Done".to_owned()],
                    vec!["TUI".to_owned(), "In progress".to_owned()]
                ]
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
            vec![Block::Paragraph(
                "See ![Architecture](docs/arch.png) for details.".to_owned()
            )]
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
