use iced::{
    Element, Fill, Padding,
    widget::{Column, column, container, scrollable, text},
};
use paperview_core::parser::{ParsedDocument, TocItem};

use crate::theme;

pub fn view<Message: 'static>(
    document: &ParsedDocument,
    active_block_index: Option<usize>,
) -> Element<'_, Message> {
    container(scrollable(toc_content(document.toc(), active_block_index)))
        .width(280)
        .height(Fill)
        .padding([22, 18])
        .style(|_| theme::navigation_container())
        .into()
}

fn toc_content<Message: 'static>(
    toc: Vec<TocItem>,
    active_block_index: Option<usize>,
) -> Column<'static, Message> {
    let mut content = column![text("On this page").size(14).color(theme::SHELL_TEXT)].spacing(10);

    if toc.is_empty() {
        return content.push(text("No headings").size(12).color(theme::SHELL_TEXT_MUTED));
    }

    for item in toc {
        let is_active = active_block_index == Some(item.block_index);
        content = content.push(toc_item(item, is_active));
    }

    content
}

fn toc_item<Message: 'static>(item: TocItem, is_active: bool) -> Element<'static, Message> {
    let indent = f32::from(item.level.as_depth().saturating_sub(1)) * 12.0;

    container(
        text(item.title)
            .size(12)
            .color(if is_active {
                theme::SHELL_ACCENT
            } else {
                theme::SHELL_TEXT_MUTED
            })
            .width(Fill),
    )
    .padding(Padding {
        top: 3.0,
        right: 0.0,
        bottom: 3.0,
        left: indent,
    })
    .width(Fill)
    .into()
}
