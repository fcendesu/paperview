use iced::{
    Element, Fill, Padding,
    widget::{Column, button, column, container, scrollable, text},
};
use paperview_core::parser::{ParsedDocument, TocItem};

use crate::theme;

pub fn view<Message: Clone + 'static>(
    document: &ParsedDocument,
    active_block_index: Option<usize>,
    on_select: impl Fn(usize) -> Message + Copy + 'static,
) -> Element<'_, Message> {
    container(scrollable(toc_content(
        document.toc(),
        active_block_index,
        on_select,
    )))
    .width(280)
    .height(Fill)
    .padding([22, 18])
    .style(|_| theme::navigation_container())
    .into()
}

fn toc_content<Message: Clone + 'static>(
    toc: Vec<TocItem>,
    active_block_index: Option<usize>,
    on_select: impl Fn(usize) -> Message + Copy + 'static,
) -> Column<'static, Message> {
    let mut content = column![text("On this page").size(14).color(theme::SHELL_TEXT)].spacing(10);

    if toc.is_empty() {
        return content.push(text("No headings").size(12).color(theme::SHELL_TEXT_MUTED));
    }

    for item in toc {
        let is_active = active_block_index == Some(item.block_index);
        content = content.push(toc_item(item, is_active, on_select));
    }

    content
}

fn toc_item<Message: Clone + 'static>(
    item: TocItem,
    is_active: bool,
    on_select: impl Fn(usize) -> Message + Copy + 'static,
) -> Element<'static, Message> {
    let indent = f32::from(item.level.as_depth().saturating_sub(1)) * 12.0;

    button(text(item.title).size(12).width(Fill))
        .padding(Padding {
            top: 3.0,
            right: 0.0,
            bottom: 3.0,
            left: indent,
        })
        .width(Fill)
        .style(move |_, status| theme::toc_item_button(is_active, status))
        .on_press(on_select(item.block_index))
        .into()
}
