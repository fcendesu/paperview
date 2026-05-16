use iced::{
    Element, Fill,
    widget::{Column, column, container, rule, scrollable, text},
};
use paperview_core::{
    Document,
    parser::{Block, HeadingLevel},
};

use crate::theme;

pub fn view<Message: 'static>(document: &Document) -> Element<'_, Message> {
    let mut content = column![].spacing(18).width(Fill);

    for block in &document.parsed().blocks {
        content = content.push(block_view(block));
    }

    container(scrollable(
        container(content)
            .padding([48, 56])
            .max_width(860)
            .style(|_| theme::paper_container()),
    ))
    .width(Fill)
    .height(Fill)
    .center_x(Fill)
    .padding([28, 0])
    .style(|_| theme::reader_backdrop())
    .into()
}

fn block_view<Message: 'static>(block: &Block) -> Element<'_, Message> {
    match block {
        Block::Heading { level, text } => heading(*level, text),
        Block::Paragraph(text) => paragraph(text),
        Block::BlockQuote(text) => blockquote(text),
        Block::CodeBlock { language, code } => code_block(language.as_deref(), code),
        Block::List { ordered, items } => list(*ordered, items),
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
