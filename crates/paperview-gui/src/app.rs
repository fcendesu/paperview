use std::{ffi::OsString, path::PathBuf};

use iced::{
    Element, Fill, Never,
    widget::{column, container, row, text},
};
use paperview_core::Document;

use crate::{reader, theme};

#[derive(Debug, Clone)]
pub struct PaperView {
    document: Option<Document>,
    status: Status,
}

#[derive(Debug, Clone)]
enum Status {
    Empty,
    Loaded(PathBuf),
    Error(String),
}

impl PaperView {
    #[must_use]
    pub fn from_args(args: impl IntoIterator<Item = OsString>) -> Self {
        let args = args.into_iter().collect::<Vec<_>>();

        match args.as_slice() {
            [] => Self {
                document: None,
                status: Status::Empty,
            },
            [path] => {
                let path = PathBuf::from(path);

                match Document::open(&path) {
                    Ok(document) => Self {
                        document: Some(document),
                        status: Status::Loaded(path),
                    },
                    Err(error) => Self {
                        document: None,
                        status: Status::Error(error.to_string()),
                    },
                }
            }
            _ => Self {
                document: None,
                status: Status::Error("usage: paperview-gui [file]".to_owned()),
            },
        }
    }
}

pub fn title(state: &PaperView) -> String {
    state.document.as_ref().map_or_else(
        || "PaperView".to_owned(),
        |document| format!("{} - PaperView", document.title()),
    )
}

pub fn iced_theme(_state: &PaperView) -> iced::Theme {
    iced::Theme::Dark
}

pub fn style(_state: &PaperView, _theme: &iced::Theme) -> iced::theme::Style {
    theme::application_style()
}

pub fn view(state: &PaperView) -> Element<'_, Never> {
    let header = header(state);
    let tab_bar = tab_bar(state);
    let body = match &state.document {
        Some(document) => reader::view(document),
        None => empty_state(&state.status),
    };

    container(column![header, tab_bar, body].height(Fill))
        .width(Fill)
        .height(Fill)
        .style(|_| theme::shell_container())
        .into()
}

fn header(state: &PaperView) -> Element<'_, Never> {
    let subtitle = match &state.status {
        Status::Empty => "No document open".to_owned(),
        Status::Loaded(path) => path.display().to_string(),
        Status::Error(error) => error.clone(),
    };

    container(
        row![
            column![
                text("PaperView").size(18).color(theme::SHELL_TEXT),
                text(subtitle).size(12).color(theme::SHELL_TEXT_MUTED)
            ]
            .spacing(4)
        ]
        .height(64),
    )
    .padding([14, 18])
    .width(Fill)
    .style(|_| theme::header_container())
    .into()
}

fn tab_bar(state: &PaperView) -> Element<'_, Never> {
    let content = match &state.document {
        Some(document) => {
            let path = document
                .path()
                .map_or_else(|| "<memory>".to_owned(), |path| path.display().to_string());

            container(
                column![
                    text(document.title()).size(14).color(theme::READER_TEXT),
                    text(path).size(11).color(theme::READER_TEXT_MUTED)
                ]
                .spacing(2),
            )
            .padding([8, 14])
            .width(360)
            .style(|_| theme::active_tab_container())
        }
        None => container(text("No file").size(13).color(theme::SHELL_TEXT_MUTED))
            .padding([8, 14])
            .width(160)
            .style(|_| theme::inactive_tab_container()),
    };

    container(row![content].height(48))
        .padding([8, 18])
        .width(Fill)
        .style(|_| theme::tab_bar_container())
        .into()
}

fn empty_state(status: &Status) -> Element<'_, Never> {
    let (title, detail) = match status {
        Status::Empty => (
            "Open a Markdown file",
            "Launch with paperview-gui <file> to preview the native reader shell.",
        ),
        Status::Loaded(_) => ("Document loaded", ""),
        Status::Error(error) => ("Could not open document", error.as_str()),
    };

    container(
        container(
            column![
                text(title).size(28).color(theme::READER_TEXT),
                text(detail).size(16).color(theme::READER_TEXT_MUTED)
            ]
            .spacing(12),
        )
        .padding(48)
        .max_width(760)
        .style(|_| theme::paper_container()),
    )
    .width(Fill)
    .height(Fill)
    .center(Fill)
    .style(|_| theme::reader_backdrop())
    .into()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{PaperView, title};

    #[test]
    fn empty_window_title_is_app_name() {
        let state = PaperView::from_args([]);

        assert_eq!(title(&state), "PaperView");
    }

    #[test]
    fn too_many_args_keeps_app_open_with_error_state() {
        let state = PaperView::from_args([OsString::from("one.md"), OsString::from("two.md")]);

        assert_eq!(title(&state), "PaperView");
    }
}
