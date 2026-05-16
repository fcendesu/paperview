use iced::{
    Element, Fill, Never,
    widget::{Column, column, container, scrollable, text},
};
use paperview_core::{FileEntry, History};

use crate::theme;

pub fn view(history: &History) -> Element<'_, Never> {
    container(scrollable(content(history)))
        .width(280)
        .height(Fill)
        .padding([22, 18])
        .style(|_| theme::history_container())
        .into()
}

fn content(history: &History) -> Column<'_, Never> {
    let mut content = column![text("History").size(14).color(theme::SHELL_TEXT)].spacing(12);

    if history.is_empty() {
        return content.push(
            text("No recent files")
                .size(12)
                .color(theme::SHELL_TEXT_MUTED),
        );
    }

    for entry in history.entries() {
        content = content.push(history_item(entry));
    }

    content
}

fn history_item(entry: &FileEntry) -> Element<'_, Never> {
    container(
        column![
            text(entry.title()).size(13).color(theme::SHELL_TEXT),
            text(entry.path().display().to_string())
                .size(11)
                .color(theme::SHELL_TEXT_MUTED)
        ]
        .spacing(3),
    )
    .padding([9, 10])
    .width(Fill)
    .style(|_| theme::active_history_item_container())
    .into()
}
