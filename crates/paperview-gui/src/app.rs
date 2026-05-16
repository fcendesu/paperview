use std::{ffi::OsString, path::PathBuf, sync::mpsc};

use iced::{
    Element, Event, Fill, Subscription,
    event::{self, Status as EventStatus},
    futures::{SinkExt, StreamExt, stream::BoxStream},
    keyboard,
    widget::{column, container, row, text},
    window,
};
use paperview_core::{Document, History, HistoryStore, WatchEvent, watch_file};

use crate::{history, navigation, reader, theme};

#[derive(Debug, Clone)]
pub struct PaperView {
    document: Option<Document>,
    history: History,
    history_store: HistoryStore,
    status: Status,
    is_drag_hovered: bool,
    is_zen: bool,
}

#[derive(Debug, Clone)]
enum Status {
    Empty,
    Loaded(PathBuf),
    Hovering(PathBuf),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenHistory(PathBuf),
    FileChanged(PathBuf),
    WatchFailed(String),
    FileHovered(PathBuf),
    FilesHoveredLeft,
    FileDropped(PathBuf),
    ToggleZen,
}

#[derive(Debug, Clone, Hash)]
struct ActiveWatchPath(PathBuf);

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
                    is_drag_hovered: false,
                    is_zen: false,
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
                            is_drag_hovered: false,
                            is_zen: false,
                        }
                    }
                    Err(error) => Self {
                        document: None,
                        history,
                        history_store,
                        status: Status::Error(error.to_string()),
                        is_drag_hovered: false,
                        is_zen: false,
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
                    is_drag_hovered: false,
                    is_zen: false,
                }
            }
        }
    }
}

pub fn update(state: &mut PaperView, message: Message) {
    match message {
        Message::OpenHistory(path) => state.open_path(path),
        Message::FileChanged(path) => state.reload_path(path),
        Message::WatchFailed(error) => {
            state.status = Status::Error(error);
        }
        Message::FileHovered(path) => {
            state.is_drag_hovered = true;
            state.status = Status::Hovering(path);
        }
        Message::FilesHoveredLeft => {
            state.is_drag_hovered = false;
            if matches!(state.status, Status::Hovering(_)) {
                state.status = state.active_path().map_or(Status::Empty, Status::Loaded);
            }
        }
        Message::FileDropped(path) => {
            state.is_drag_hovered = false;
            state.open_path(path);
        }
        Message::ToggleZen => {
            state.is_zen = !state.is_zen;
        }
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

    fn reload_path(&mut self, path: PathBuf) {
        if !self.is_active_path(&path) {
            return;
        }

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

    fn active_path(&self) -> Option<PathBuf> {
        self.document
            .as_ref()
            .and_then(Document::path)
            .map(PathBuf::from)
    }

    fn is_active_path(&self, path: &PathBuf) -> bool {
        self.document.as_ref().and_then(Document::path) == Some(path)
    }
}

pub fn subscription(state: &PaperView) -> Subscription<Message> {
    let runtime_events = event::listen_with(runtime_event);
    let file_watch = state.active_path().map_or_else(Subscription::none, |path| {
        Subscription::run_with(ActiveWatchPath(path), watch_active_document)
    });

    Subscription::batch([runtime_events, file_watch])
}

fn runtime_event(event: Event, _status: EventStatus, _window: window::Id) -> Option<Message> {
    match event {
        Event::Window(window::Event::FileHovered(path)) => Some(Message::FileHovered(path)),
        Event::Window(window::Event::FilesHoveredLeft) => Some(Message::FilesHoveredLeft),
        Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.command()
            && modifiers.shift()
            && key
                .to_latin(physical_key)
                .is_some_and(|character| character.eq_ignore_ascii_case(&'f')) =>
        {
            Some(Message::ToggleZen)
        }
        _ => None,
    }
}

fn watch_active_document(path: &ActiveWatchPath) -> BoxStream<'static, Message> {
    let path = path.0.clone();

    iced::stream::channel(32, async move |mut output| {
        let (sender, receiver) = mpsc::channel();
        let _watcher = match watch_file(&path, sender) {
            Ok(watcher) => watcher,
            Err(error) => {
                let _ = output.send(Message::WatchFailed(error.to_string())).await;
                return;
            }
        };

        while let Ok(event) = receiver.recv() {
            match event {
                WatchEvent::Changed(path) => {
                    if output.send(Message::FileChanged(path)).await.is_err() {
                        break;
                    }
                }
            }
        }
    })
    .boxed()
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
    let body = match &state.document {
        Some(document) if state.is_zen => reader::view(document),
        Some(document) => row![
            history::view(&state.history),
            reader::view(document),
            navigation::view(document.parsed())
        ]
        .into(),
        None => empty_state(&state.status),
    };
    let layout = if state.is_zen {
        column![header, body].height(Fill)
    } else {
        column![header, tab_bar(state), body].height(Fill)
    };

    container(layout)
        .width(Fill)
        .height(Fill)
        .style(|_| theme::shell_container(state.is_drag_hovered))
        .into()
}

fn header(state: &PaperView) -> Element<'_, Message> {
    let subtitle = match &state.status {
        Status::Empty => format!(
            "No document open - {}",
            state.history_store.path().display()
        ),
        Status::Loaded(path) => path.display().to_string(),
        Status::Hovering(path) => format!("Drop to open {}", path.display()),
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
        Status::Hovering(_) => (
            "Drop to open",
            "Release the file to preview it in PaperView.",
        ),
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
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use iced::{
        Event,
        keyboard::{
            Key, Location, Modifiers,
            key::{Code, Physical},
        },
    };
    use paperview_core::HistoryStore;

    use super::{Message, PaperView, runtime_event, title, update};

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

    #[test]
    fn file_changed_reloads_active_document() {
        let path = temp_doc("live-reload.md", "# Before\n\nInitial body.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("reload.toml"));

        fs::write(&path, "# After\n\nUpdated body.").expect("rewrite test document");
        update(&mut state, Message::FileChanged(path.clone()));

        assert_eq!(
            state.document.as_ref().map(|document| document.title()),
            Some("After")
        );
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn dropped_file_opens_document() {
        let path = temp_doc("dropped.md", "# Dropped\n\nOpened from drop.");
        let mut state = PaperView::from_args_with_store([], temp_store("drop.toml"));

        update(&mut state, Message::FileDropped(path.clone()));

        assert_eq!(
            state.document.as_ref().map(|document| document.title()),
            Some("Dropped")
        );
        assert!(!state.is_drag_hovered);
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn hover_leave_restores_loaded_status() {
        let path = temp_doc("active.md", "# Active\n\nCurrent.");
        let hover_path = temp_doc("hovered.md", "# Hovered\n\nMaybe next.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("hover.toml"));

        update(&mut state, Message::FileHovered(hover_path.clone()));
        assert!(state.is_drag_hovered);
        assert!(matches!(state.status, super::Status::Hovering(_)));

        update(&mut state, Message::FilesHoveredLeft);

        assert!(!state.is_drag_hovered);
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove active test document");
        fs::remove_file(hover_path).expect("remove hovered test document");
    }

    #[test]
    fn toggle_zen_flips_layout_state() {
        let mut state = PaperView::from_args_with_store([], temp_store("zen.toml"));

        update(&mut state, Message::ToggleZen);
        assert!(state.is_zen);

        update(&mut state, Message::ToggleZen);
        assert!(!state.is_zen);
    }

    #[test]
    fn command_shift_f_maps_to_zen_toggle() {
        let message = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("f".into()),
                modified_key: Key::Character("F".into()),
                physical_key: Physical::Code(Code::KeyF),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND | Modifiers::SHIFT,
                text: Some("F".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );

        assert!(matches!(message, Some(Message::ToggleZen)));
    }

    fn temp_store(name: &str) -> HistoryStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        HistoryStore::new(std::env::temp_dir().join(format!("paperview-gui-{nanos}-{name}")))
    }

    fn temp_doc(name: &str, source: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("paperview-gui-{nanos}-{name}"));

        fs::write(&path, source).expect("write test document");

        path
    }
}
