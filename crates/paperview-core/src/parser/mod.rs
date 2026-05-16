pub mod elements;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

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
    List {
        ordered: bool,
        items: Vec<String>,
    },
    Rule,
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

#[derive(Debug, Default)]
struct DocumentBuilder {
    blocks: Vec<Block>,
    open_block: Option<OpenBlock>,
    open_list: Option<OpenList>,
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
            Event::SoftBreak | Event::HardBreak => self.push_text("\n"),
            Event::Rule => self.blocks.push(Block::Rule),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.open_block = Some(OpenBlock::Heading {
                    level,
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

    fn close_block(&mut self) {
        let Some(open_block) = self.open_block.take() else {
            return;
        };

        match open_block {
            OpenBlock::Heading { level, text } => self.blocks.push(Block::Heading {
                level,
                text: normalize_text(&text),
            }),
            OpenBlock::Paragraph(text) => self.blocks.push(Block::Paragraph(normalize_text(&text))),
            OpenBlock::BlockQuote(text) => {
                self.blocks.push(Block::BlockQuote(normalize_text(&text)));
            }
            OpenBlock::CodeBlock { language, code } => {
                self.blocks.push(Block::CodeBlock { language, code });
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

#[cfg(test)]
mod tests {
    use pulldown_cmark::HeadingLevel;

    use super::{Block, parse_markdown};

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
}
