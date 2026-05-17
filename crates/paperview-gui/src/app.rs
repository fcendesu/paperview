use std::{ffi::OsString, path::PathBuf, sync::mpsc};

use iced::{
    Element, Event, Fill, Length, Subscription,
    event::{self, Status as EventStatus},
    futures::{SinkExt, StreamExt, stream::BoxStream},
    keyboard,
    widget::{button, column, container, row, text},
    window,
};
use paperview_core::{Document, History, HistoryStore, OpenDocuments, WatchEvent, watch_file};

use crate::{history, navigation, reader, theme};

const DEFAULT_SPLIT_PRIMARY_WIDTH: u16 = 50;
const MIN_SPLIT_PRIMARY_WIDTH: u16 = 30;
const MAX_SPLIT_PRIMARY_WIDTH: u16 = 70;
const SPLIT_RESIZE_STEP: u16 = 10;

#[derive(Debug, Clone)]
pub struct PaperView {
    documents: OpenDocuments,
    history: History,
    history_store: HistoryStore,
    status: Status,
    is_drag_hovered: bool,
    is_zen: bool,
    split_document_index: Option<usize>,
    split_primary_width: u16,
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
    OpenDroppedFiles(Vec<PathBuf>),
    ToggleZen,
    ToggleSplit,
    ResizeSplit(SplitResize),
    SelectSplitTab(usize),
    SelectTab(usize),
    CloseTab(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum SplitResize {
    GrowPrimary,
    ShrinkPrimary,
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
                    documents: OpenDocuments::new(),
                    history,
                    history_store,
                    status: Status::Empty,
                    is_drag_hovered: false,
                    is_zen: false,
                    split_document_index: None,
                    split_primary_width: DEFAULT_SPLIT_PRIMARY_WIDTH,
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
                            documents: OpenDocuments::from_document(document),
                            history,
                            history_store,
                            status: Status::Loaded(path),
                            is_drag_hovered: false,
                            is_zen: false,
                            split_document_index: None,
                            split_primary_width: DEFAULT_SPLIT_PRIMARY_WIDTH,
                        }
                    }
                    Err(error) => Self {
                        documents: OpenDocuments::new(),
                        history,
                        history_store,
                        status: Status::Error(error.to_string()),
                        is_drag_hovered: false,
                        is_zen: false,
                        split_document_index: None,
                        split_primary_width: DEFAULT_SPLIT_PRIMARY_WIDTH,
                    },
                }
            }
            _ => {
                let history = load_history(&history_store);

                Self {
                    documents: OpenDocuments::new(),
                    history,
                    history_store,
                    status: Status::Error("usage: paperview-gui [file]".to_owned()),
                    is_drag_hovered: false,
                    is_zen: false,
                    split_document_index: None,
                    split_primary_width: DEFAULT_SPLIT_PRIMARY_WIDTH,
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
        Message::OpenDroppedFiles(paths) => {
            state.is_drag_hovered = false;
            state.open_dropped_files(paths);
        }
        Message::ToggleZen => {
            state.is_zen = !state.is_zen;
        }
        Message::ToggleSplit => state.toggle_split(),
        Message::ResizeSplit(direction) => state.resize_split(direction),
        Message::SelectSplitTab(index) => state.select_split_tab(index),
        Message::SelectTab(index) => state.select_tab(index),
        Message::CloseTab(index) => state.close_tab(index),
    }
}

impl PaperView {
    fn open_path(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(document) => {
                self.history.record_document(&document);
                save_history(&self.history_store, &self.history);
                self.documents.open_or_activate(document);
                self.status = Status::Loaded(path);
                self.ensure_split_target();
            }
            Err(error) => {
                self.status = Status::Error(error.to_string());
            }
        }
    }

    fn open_dropped_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        let mut last_error = None;

        for path in paths {
            match Document::open(&path) {
                Ok(document) => {
                    self.history.record_document(&document);
                    save_history(&self.history_store, &self.history);
                    self.documents.open_or_activate(document);
                    self.status = Status::Loaded(path);
                    self.ensure_split_target();
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                }
            }
        }

        if let Some(error) = last_error {
            self.status = Status::Error(error);
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
                self.documents.replace_active(document);
                self.status = Status::Loaded(path);
            }
            Err(error) => {
                self.status = Status::Error(error.to_string());
            }
        }
    }

    fn active_path(&self) -> Option<PathBuf> {
        self.documents
            .active()
            .and_then(Document::path)
            .map(PathBuf::from)
    }

    fn is_active_path(&self, path: &PathBuf) -> bool {
        self.documents.active().and_then(Document::path) == Some(path)
    }

    fn select_tab(&mut self, index: usize) {
        self.documents.select(index);
        self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
        self.ensure_split_target();
    }

    fn close_tab(&mut self, index: usize) {
        self.documents.close(index);
        self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
        self.ensure_split_target();
    }

    fn select_split_tab(&mut self, index: usize) {
        if self.split_document_index.is_some()
            && Some(index) != self.documents.active_index()
            && index < self.documents.len()
        {
            self.split_document_index = Some(index);
        }
    }

    fn toggle_split(&mut self) {
        self.split_document_index = self
            .split_document_index
            .is_none()
            .then(|| self.first_secondary_index())
            .flatten();
    }

    fn resize_split(&mut self, direction: SplitResize) {
        if self.split_document_index.is_none() {
            return;
        }

        self.split_primary_width = match direction {
            SplitResize::GrowPrimary => self
                .split_primary_width
                .saturating_add(SPLIT_RESIZE_STEP)
                .min(MAX_SPLIT_PRIMARY_WIDTH),
            SplitResize::ShrinkPrimary => self
                .split_primary_width
                .saturating_sub(SPLIT_RESIZE_STEP)
                .max(MIN_SPLIT_PRIMARY_WIDTH),
        };
    }

    fn ensure_split_target(&mut self) {
        if self.split_document_index.is_some_and(|index| {
            Some(index) == self.documents.active_index() || index >= self.documents.len()
        }) {
            self.split_document_index = self.first_secondary_index();
        }
    }

    fn first_secondary_index(&self) -> Option<usize> {
        let active = self.documents.active_index()?;

        self.documents
            .iter()
            .map(|(index, _)| index)
            .find(|index| *index != active)
    }

    fn split_document(&self) -> Option<&Document> {
        let split_index = self.split_document_index?;

        self.documents
            .iter()
            .find_map(|(index, document)| (index == split_index).then_some(document))
    }

    fn split_widths(&self) -> (u16, u16) {
        let primary = self
            .split_primary_width
            .clamp(MIN_SPLIT_PRIMARY_WIDTH, MAX_SPLIT_PRIMARY_WIDTH);

        (primary, 100 - primary)
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
        Event::Window(window::Event::FileDropped(path)) => {
            Some(Message::OpenDroppedFiles(vec![path]))
        }
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
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.command()
            && key
                .to_latin(physical_key)
                .is_some_and(|character| character.eq_ignore_ascii_case(&'\\')) =>
        {
            Some(Message::ToggleSplit)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.command()
            && key
                .to_latin(physical_key)
                .is_some_and(|character| character == ']') =>
        {
            Some(Message::ResizeSplit(SplitResize::GrowPrimary))
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.command()
            && key
                .to_latin(physical_key)
                .is_some_and(|character| character == '[') =>
        {
            Some(Message::ResizeSplit(SplitResize::ShrinkPrimary))
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
    state.documents.active().map_or_else(
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
    let body = match state.documents.active() {
        Some(document) if state.is_zen => reader::view(document),
        Some(document) => {
            let reader = if let Some(secondary) = state.split_document() {
                let (primary_width, secondary_width) = state.split_widths();

                row![
                    container(reader::view(document)).width(Length::FillPortion(primary_width)),
                    container(reader::view(secondary)).width(Length::FillPortion(secondary_width))
                ]
                .spacing(1)
                .into()
            } else {
                reader::view(document)
            };

            row![
                history::view(&state.history),
                reader,
                navigation::view(document.parsed())
            ]
            .into()
        }
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

    let is_split_enabled = state.split_document_index.is_some();
    let split_label = if is_split_enabled {
        let (primary, secondary) = state.split_widths();
        format!("Split {primary}/{secondary}")
    } else {
        "Split".to_owned()
    };
    let split_button = button(text(split_label).size(13))
        .padding([7, 12])
        .style(move |_, status| theme::header_action_button(is_split_enabled, status));
    let split_button = if state.documents.len() > 1 {
        split_button.on_press(Message::ToggleSplit)
    } else {
        split_button
    };

    container(
        row![
            column![
                text("PaperView").size(18).color(theme::SHELL_TEXT),
                text(subtitle).size(12).color(theme::SHELL_TEXT_MUTED)
            ]
            .spacing(4)
            .width(Fill),
            split_button
        ]
        .height(64),
    )
    .padding([14, 18])
    .width(Fill)
    .style(|_| theme::header_container())
    .into()
}

fn tab_bar(state: &PaperView) -> Element<'_, Message> {
    let mut tabs = row![].height(48).spacing(8);

    if state.documents.is_empty() {
        tabs = tabs.push(
            container(text("No file").size(13).color(theme::SHELL_TEXT_MUTED))
                .padding([8, 14])
                .width(160)
                .style(|_| theme::inactive_tab_container()),
        );
    } else {
        for (index, document) in state.documents.iter() {
            let path = document
                .path()
                .map_or_else(|| "<memory>".to_owned(), |path| path.display().to_string());
            let is_active = state.documents.active_index() == Some(index);
            let is_secondary = state.split_document_index == Some(index);

            let mut tab_content = row![
                column![
                    text(document.title()).size(14).color(if is_active {
                        theme::READER_TEXT
                    } else {
                        theme::SHELL_TEXT
                    }),
                    text(path).size(11).color(if is_active {
                        theme::READER_TEXT_MUTED
                    } else {
                        theme::SHELL_TEXT_MUTED
                    })
                ]
                .spacing(2)
                .width(Fill)
            ]
            .spacing(8);

            if state.split_document_index.is_some() && !is_active {
                let split_label = if is_secondary { "Side" } else { "Use" };
                tab_content = tab_content.push(
                    button(text(split_label).size(11))
                        .padding([2, 7])
                        .style(move |_, status| theme::split_tab_button(is_secondary, status))
                        .on_press(Message::SelectSplitTab(index)),
                );
            }

            tab_content = tab_content.push(
                button(text("x").size(13))
                    .padding([1, 6])
                    .style(move |_, status| theme::tab_close_button(is_active, status))
                    .on_press(Message::CloseTab(index)),
            );

            let tab = button(tab_content)
                .padding([8, 14])
                .width(360)
                .style(move |_, status| theme::tab_button(is_active, status))
                .on_press(Message::SelectTab(index));

            tabs = tabs.push(tab);
        }
    }

    container(tabs)
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
    use paperview_core::{Document, HistoryStore};

    use super::{Message, PaperView, SplitResize, runtime_event, title, update};

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
            state.documents.active().map(Document::title),
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

        assert_eq!(state.documents.active().map(Document::title), Some("After"));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn dropped_file_opens_document() {
        let path = temp_doc("dropped.md", "# Dropped\n\nOpened from drop.");
        let mut state = PaperView::from_args_with_store([], temp_store("drop.toml"));

        update(&mut state, Message::OpenDroppedFiles(vec![path.clone()]));

        assert_eq!(
            state.documents.active().map(Document::title),
            Some("Dropped")
        );
        assert!(!state.is_drag_hovered);
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn opening_second_file_adds_active_tab() {
        let first = temp_doc("first-tab.md", "# First\n\nOne.");
        let second = temp_doc("second-tab.md", "# Second\n\nTwo.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&first)], temp_store("tabs.toml"));

        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        assert_eq!(state.documents.len(), 2);
        assert_eq!(state.documents.active_index(), Some(1));
        assert_eq!(
            state.documents.active().map(Document::title),
            Some("Second")
        );

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn dropped_file_batch_opens_each_supported_file_as_tab() {
        let first = temp_doc("batch-first.md", "# First\n\nOne.");
        let second = temp_doc("batch-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store([], temp_store("batch.toml"));

        update(
            &mut state,
            Message::OpenDroppedFiles(vec![first.clone(), second.clone()]),
        );

        assert_eq!(state.documents.len(), 2);
        assert_eq!(state.documents.active_index(), Some(1));
        assert_eq!(
            state.documents.active().map(Document::title),
            Some("Second")
        );

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn dropped_file_batch_keeps_supported_tabs_when_one_file_fails() {
        let supported = temp_doc("batch-supported.md", "# Supported\n\nGood.");
        let unsupported = temp_doc("batch-unsupported.html", "<h1>Nope</h1>");
        let mut state = PaperView::from_args_with_store([], temp_store("batch-error.toml"));

        update(
            &mut state,
            Message::OpenDroppedFiles(vec![supported.clone(), unsupported.clone()]),
        );

        assert_eq!(state.documents.len(), 1);
        assert_eq!(
            state.documents.active().map(Document::title),
            Some("Supported")
        );
        assert!(matches!(state.status, super::Status::Error(_)));

        fs::remove_file(supported).expect("remove supported test document");
        fs::remove_file(unsupported).expect("remove unsupported test document");
    }

    #[test]
    fn selecting_tab_changes_active_document() {
        let first = temp_doc("select-first.md", "# First\n\nOne.");
        let second = temp_doc("select-second.md", "# Second\n\nTwo.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&first)], temp_store("select.toml"));
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        update(&mut state, Message::SelectTab(0));

        assert_eq!(state.documents.active_index(), Some(0));
        assert_eq!(state.documents.active().map(Document::title), Some("First"));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &first)
        );

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn closing_active_tab_selects_neighboring_tab() {
        let first = temp_doc("close-first.md", "# First\n\nOne.");
        let second = temp_doc("close-second.md", "# Second\n\nTwo.");
        let third = temp_doc("close-third.md", "# Third\n\nThree.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&first)], temp_store("close.toml"));
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        update(&mut state, Message::OpenDroppedFiles(vec![third.clone()]));
        update(&mut state, Message::SelectTab(1));

        update(&mut state, Message::CloseTab(1));

        assert_eq!(state.documents.len(), 2);
        assert_eq!(state.documents.active_index(), Some(1));
        assert_eq!(state.documents.active().map(Document::title), Some("Third"));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &third)
        );

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
        fs::remove_file(third).expect("remove third test document");
    }

    #[test]
    fn closing_last_tab_returns_to_empty_state() {
        let path = temp_doc("close-last.md", "# Last\n\nDone.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("close-last.toml"));

        update(&mut state, Message::CloseTab(0));

        assert!(state.documents.is_empty());
        assert_eq!(state.documents.active_index(), None);
        assert!(matches!(state.status, super::Status::Empty));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn split_toggle_needs_secondary_tab() {
        let path = temp_doc("split-single.md", "# Single\n\nOnly.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("split-single.toml"),
        );

        update(&mut state, Message::ToggleSplit);

        assert_eq!(state.split_document_index, None);
        assert!(state.split_document().is_none());

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn split_toggle_targets_first_non_active_tab() {
        let first = temp_doc("split-first.md", "# First\n\nOne.");
        let second = temp_doc("split-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-toggle.toml"),
        );
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        update(&mut state, Message::ToggleSplit);

        assert_eq!(state.documents.active_index(), Some(1));
        assert_eq!(state.split_document_index, Some(0));
        assert_eq!(state.split_document().map(Document::title), Some("First"));

        update(&mut state, Message::ToggleSplit);

        assert_eq!(state.split_document_index, None);

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn split_resize_changes_primary_width_when_split_is_enabled() {
        let first = temp_doc("split-resize-first.md", "# First\n\nOne.");
        let second = temp_doc("split-resize-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-resize.toml"),
        );
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        update(&mut state, Message::ToggleSplit);

        update(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));

        assert_eq!(state.split_widths(), (60, 40));

        update(&mut state, Message::ResizeSplit(SplitResize::ShrinkPrimary));
        update(&mut state, Message::ResizeSplit(SplitResize::ShrinkPrimary));

        assert_eq!(state.split_widths(), (40, 60));

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn split_resize_is_bounded_and_requires_enabled_split() {
        let first = temp_doc("split-bounds-first.md", "# First\n\nOne.");
        let second = temp_doc("split-bounds-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-bounds.toml"),
        );
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        update(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));
        assert_eq!(state.split_widths(), (50, 50));

        update(&mut state, Message::ToggleSplit);
        for _ in 0..8 {
            update(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));
        }
        assert_eq!(state.split_widths(), (70, 30));

        for _ in 0..8 {
            update(&mut state, Message::ResizeSplit(SplitResize::ShrinkPrimary));
        }
        assert_eq!(state.split_widths(), (30, 70));

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn selecting_split_tab_retargets_secondary_document() {
        let first = temp_doc("split-select-first.md", "# First\n\nOne.");
        let second = temp_doc("split-select-second.md", "# Second\n\nTwo.");
        let third = temp_doc("split-select-third.md", "# Third\n\nThree.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-select.toml"),
        );
        update(
            &mut state,
            Message::OpenDroppedFiles(vec![second.clone(), third.clone()]),
        );
        update(&mut state, Message::ToggleSplit);

        update(&mut state, Message::SelectTab(0));

        assert_eq!(state.documents.active_index(), Some(0));
        assert_eq!(state.split_document_index, Some(1));
        assert_eq!(state.split_document().map(Document::title), Some("Second"));

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
        fs::remove_file(third).expect("remove third test document");
    }

    #[test]
    fn choosing_secondary_tab_updates_split_document() {
        let first = temp_doc("split-choice-first.md", "# First\n\nOne.");
        let second = temp_doc("split-choice-second.md", "# Second\n\nTwo.");
        let third = temp_doc("split-choice-third.md", "# Third\n\nThree.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-choice.toml"),
        );
        update(
            &mut state,
            Message::OpenDroppedFiles(vec![second.clone(), third.clone()]),
        );
        update(&mut state, Message::ToggleSplit);

        update(&mut state, Message::SelectSplitTab(1));

        assert_eq!(state.documents.active_index(), Some(2));
        assert_eq!(state.split_document_index, Some(1));
        assert_eq!(state.split_document().map(Document::title), Some("Second"));

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
        fs::remove_file(third).expect("remove third test document");
    }

    #[test]
    fn choosing_active_tab_as_secondary_is_ignored() {
        let first = temp_doc("split-active-first.md", "# First\n\nOne.");
        let second = temp_doc("split-active-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-active.toml"),
        );
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        update(&mut state, Message::ToggleSplit);

        update(&mut state, Message::SelectSplitTab(1));

        assert_eq!(state.documents.active_index(), Some(1));
        assert_eq!(state.split_document_index, Some(0));

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn closing_split_tab_disables_split_when_no_secondary_remains() {
        let first = temp_doc("split-close-first.md", "# First\n\nOne.");
        let second = temp_doc("split-close-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-close.toml"),
        );
        update(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        update(&mut state, Message::ToggleSplit);

        update(&mut state, Message::CloseTab(0));

        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.documents.active_index(), Some(0));
        assert_eq!(state.split_document_index, None);

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
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

    #[test]
    fn command_backslash_maps_to_split_toggle() {
        let message = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("\\".into()),
                modified_key: Key::Character("\\".into()),
                physical_key: Physical::Code(Code::Backslash),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND,
                text: Some("\\".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );

        assert!(matches!(message, Some(Message::ToggleSplit)));
    }

    #[test]
    fn command_brackets_map_to_split_resize() {
        let grow = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("]".into()),
                modified_key: Key::Character("]".into()),
                physical_key: Physical::Code(Code::BracketRight),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND,
                text: Some("]".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );
        let shrink = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("[".into()),
                modified_key: Key::Character("[".into()),
                physical_key: Physical::Code(Code::BracketLeft),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND,
                text: Some("[".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );

        assert!(matches!(
            grow,
            Some(Message::ResizeSplit(SplitResize::GrowPrimary))
        ));
        assert!(matches!(
            shrink,
            Some(Message::ResizeSplit(SplitResize::ShrinkPrimary))
        ));
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
