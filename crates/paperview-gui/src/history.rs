use iced::{
    Element, Fill,
    widget::{Column, button, column, container, scrollable, text},
};
use paperview_core::{FileEntry, History};

use crate::{app::Message, theme};

pub fn view(history: &History) -> Element<'_, Message> {
    container(scrollable(content(history)))
        .width(280)
        .height(Fill)
        .padding([22, 18])
        .style(|_| theme::history_container())
        .into()
}

fn content(history: &History) -> Column<'_, Message> {
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

fn history_item(entry: &FileEntry) -> Element<'_, Message> {
    button(
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
    .style(|_, status| theme::history_item_button(status))
    .on_press(Message::OpenHistory(entry.path().to_path_buf()))
    .into()
}
