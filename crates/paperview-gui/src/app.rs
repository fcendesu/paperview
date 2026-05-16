use std::{ffi::OsString, path::PathBuf};

use iced::{
    Element, Fill,
    widget::{column, container, row, text},
};
use paperview_core::{Document, History, HistoryStore};

use crate::{history, navigation, reader, theme};

#[derive(Debug, Clone)]
pub struct PaperView {
    document: Option<Document>,
    history: History,
    history_store: HistoryStore,
    status: Status,
}

#[derive(Debug, Clone)]
enum Status {
    Empty,
    Loaded(PathBuf),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenHistory(PathBuf),
}

impl PaperView {
    #[must_use]
    pub fn from_args(args: impl IntoIterator<Item = OsString>) -> Self {
        Self::from_args_with_store(args, HistoryStore::default())
    }

    #[must_use]
    fn from_args_with_store(
        args: impl IntoIterator<Item = OsString>,
        history_store: HistoryStore,
    ) -> Self {
        let args = args.into_iter().collect::<Vec<_>>();

        match args.as_slice() {
            [] => {
                let history = load_history(&history_store);

                Self {
                    document: None,
                    history,
                    history_store,
                    status: Status::Empty,
                }
            }
            [path] => {
                let path = PathBuf::from(path);
                let mut history = load_history(&history_store);

                match Document::open(&path) {
                    Ok(document) => {
                        history.record_document(&document);
                        save_history(&history_store, &history);

                        Self {
                            document: Some(document),
                            history,
                            history_store,
                            status: Status::Loaded(path),
                        }
                    }
                    Err(error) => Self {
                        document: None,
                        history,
                        history_store,
                        status: Status::Error(error.to_string()),
                    },
                }
            }
            _ => {
                let history = load_history(&history_store);

                Self {
                    document: None,
                    history,
                    history_store,
                    status: Status::Error("usage: paperview-gui [file]".to_owned()),
                }
            }
        }
    }
}

pub fn update(state: &mut PaperView, message: Message) {
    match message {
        Message::OpenHistory(path) => state.open_path(path),
    }
}

impl PaperView {
    fn open_path(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(document) => {
                self.history.record_document(&document);
                save_history(&self.history_store, &self.history);
                self.document = Some(document);
                self.status = Status::Loaded(path);
            }
            Err(error) => {
                self.status = Status::Error(error.to_string());
            }
        }
    }
}

fn load_history(store: &HistoryStore) -> History {
    store.load().unwrap_or_else(|error| {
        eprintln!("{error}");
        History::new()
    })
}

fn save_history(store: &HistoryStore, history: &History) {
    if let Err(error) = store.save(history) {
        eprintln!("{error}");
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

pub fn view(state: &PaperView) -> Element<'_, Message> {
    let header = header(state);
    let tab_bar = tab_bar(state);
    let body = match &state.document {
        Some(document) => row![
            history::view(&state.history),
            reader::view(document),
            navigation::view(document.parsed())
        ]
        .into(),
        None => empty_state(&state.status),
    };

    container(column![header, tab_bar, body].height(Fill))
        .width(Fill)
        .height(Fill)
        .style(|_| theme::shell_container())
        .into()
}

fn header(state: &PaperView) -> Element<'_, Message> {
    let subtitle = match &state.status {
        Status::Empty => format!(
            "No document open - {}",
            state.history_store.path().display()
        ),
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

fn tab_bar(state: &PaperView) -> Element<'_, Message> {
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

fn empty_state(status: &Status) -> Element<'_, Message> {
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
    use std::{
        ffi::OsString,
        time::{SystemTime, UNIX_EPOCH},
    };

    use paperview_core::HistoryStore;

    use super::{Message, PaperView, title, update};

    #[test]
    fn empty_window_title_is_app_name() {
        let state = PaperView::from_args_with_store([], temp_store("empty.toml"));

        assert_eq!(title(&state), "PaperView");
    }

    #[test]
    fn too_many_args_keeps_app_open_with_error_state() {
        let state = PaperView::from_args_with_store(
            [OsString::from("one.md"), OsString::from("two.md")],
            temp_store("too-many.toml"),
        );

        assert_eq!(title(&state), "PaperView");
    }

    #[test]
    fn opening_history_path_updates_loaded_document() {
        let mut state = PaperView::from_args_with_store([], temp_store("open-history.toml"));
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/PRD.md");

        update(&mut state, Message::OpenHistory(path.clone()));

        assert_eq!(
            state.document.as_ref().map(|document| document.title()),
            Some("PaperView — Product Requirements Document (PRD)")
        );
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );
    }

    fn temp_store(name: &str) -> HistoryStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        HistoryStore::new(std::env::temp_dir().join(format!("paperview-gui-{nanos}-{name}")))
    }
}
