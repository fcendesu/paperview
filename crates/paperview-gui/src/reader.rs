use iced::{
    Element, Fill, Never,
    widget::{Column, column, container, rule, scrollable, text},
};
use paperview_core::{
    Document,
    parser::{Block, HeadingLevel},
};

use crate::theme;

pub fn view(document: &Document) -> Element<'_, Never> {
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

fn block_view(block: &Block) -> Element<'_, Never> {
    match block {
        Block::Heading { level, text } => heading(*level, text),
        Block::Paragraph(text) => paragraph(text),
        Block::BlockQuote(text) => blockquote(text),
        Block::CodeBlock { language, code } => code_block(language.as_deref(), code),
        Block::List { ordered, items } => list(*ordered, items),
        Block::Rule => rule::horizontal(1).into(),
    }
}

fn heading(level: HeadingLevel, value: &str) -> Element<'_, Never> {
    let size = match level {
        HeadingLevel::H1 => 32,
        HeadingLevel::H2 => 24,
        HeadingLevel::H3 => 20,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 18,
    };

    text(value).size(size).color(theme::READER_TEXT).into()
}

fn paragraph(value: &str) -> Element<'_, Never> {
    text(value).size(16).color(theme::READER_TEXT).into()
}

fn blockquote(value: &str) -> Element<'_, Never> {
    container(text(value).size(16).color(theme::READER_TEXT_MUTED))
        .padding([8, 14])
        .width(Fill)
        .style(|_| theme::quote_container())
        .into()
}

fn code_block<'a>(language: Option<&'a str>, code: &'a str) -> Element<'a, Never> {
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

fn list(ordered: bool, items: &[String]) -> Element<'_, Never> {
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
