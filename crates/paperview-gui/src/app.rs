use std::{ffi::OsString, path::PathBuf, process::Command, sync::mpsc};

use iced::{
    Element, Event, Fill, Length, Subscription, Task,
    event::{self, Status as EventStatus},
    futures::{SinkExt, StreamExt, stream::BoxStream},
    keyboard,
    widget::{
        button, column, container,
        operation::{self, RelativeOffset},
        row, text, text_input,
    },
    window,
};
use paperview_core::{
    Document, History, HistoryStore, OpenDocuments, SearchMatch, WatchEvent, watch_file,
};

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
    active_toc_block_index: Option<usize>,
    search_query: String,
    search_matches: Vec<SearchMatch>,
    search_selected_index: Option<usize>,
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
    ReaderScrolled(f32),
    OpenLink(String),
    LinkOpened,
    LinkOpenFailed(String),
    TocSelected(usize),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrevious,
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
                    active_toc_block_index: None,
                    search_query: String::new(),
                    search_matches: Vec::new(),
                    search_selected_index: None,
                }
            }
            [path] => {
                let path = PathBuf::from(path);
                let mut history = load_history(&history_store);

                match Document::open(&path) {
                    Ok(document) => {
                        let active_toc_block_index = first_toc_block_index(document.parsed());

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
                            active_toc_block_index,
                            search_query: String::new(),
                            search_matches: Vec::new(),
                            search_selected_index: None,
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
                        active_toc_block_index: None,
                        search_query: String::new(),
                        search_matches: Vec::new(),
                        search_selected_index: None,
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
                    active_toc_block_index: None,
                    search_query: String::new(),
                    search_matches: Vec::new(),
                    search_selected_index: None,
                }
            }
        }
    }
}

pub fn update(state: &mut PaperView, message: Message) -> Task<Message> {
    match message {
        Message::OpenHistory(path) => {
            state.open_path(path);
        }
        Message::FileChanged(path) => {
            state.reload_path(path);
        }
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
        Message::ReaderScrolled(progress) => state.sync_active_toc_to_scroll(progress),
        Message::OpenLink(target) => return state.open_link(target),
        Message::LinkOpened => {}
        Message::LinkOpenFailed(error) => {
            state.status = Status::Error(error);
        }
        Message::TocSelected(block_index) => {
            state.active_toc_block_index = Some(block_index);
            return state.scroll_to_toc_block(block_index);
        }
        Message::SearchQueryChanged(query) => {
            return state.update_search_query(query);
        }
        Message::SearchNext => return state.select_next_search_match(),
        Message::SearchPrevious => return state.select_previous_search_match(),
        Message::SelectSplitTab(index) => state.select_split_tab(index),
        Message::SelectTab(index) => state.select_tab(index),
        Message::CloseTab(index) => state.close_tab(index),
    }

    Task::none()
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
                self.sync_active_toc_to_top();
                self.refresh_search_matches();
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
                    self.sync_active_toc_to_top();
                    self.refresh_search_matches();
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
                self.sync_active_toc_to_top();
                self.refresh_search_matches();
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
        self.sync_active_toc_to_top();
        self.refresh_search_matches();
    }

    fn close_tab(&mut self, index: usize) {
        self.documents.close(index);
        self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
        self.ensure_split_target();
        self.sync_active_toc_to_top();
        self.refresh_search_matches();
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

    fn sync_active_toc_to_top(&mut self) {
        self.active_toc_block_index = self
            .documents
            .active()
            .and_then(|document| first_toc_block_index(document.parsed()));
    }

    fn sync_active_toc_to_scroll(&mut self, progress: f32) {
        let Some(document) = self.documents.active() else {
            self.active_toc_block_index = None;
            return;
        };

        self.active_toc_block_index =
            reader::active_heading_for_scroll(document.parsed(), progress);
    }

    fn scroll_to_toc_block(&self, block_index: usize) -> Task<Message> {
        let progress = self.documents.active().map_or(0.0, |document| {
            reader::heading_scroll_progress(document.parsed(), block_index)
        });

        operation::snap_to(
            reader::ACTIVE_READER_SCROLLABLE_ID,
            RelativeOffset {
                x: 0.0,
                y: progress,
            },
        )
    }

    fn open_link(&mut self, target: String) -> Task<Message> {
        if let Some(anchor) = target.strip_prefix('#') {
            return self.open_anchor_link(anchor);
        }

        let resolved =
            resolve_link_target(&target, self.documents.active().and_then(Document::path));

        Task::perform(
            async move { open_link_target(resolved) },
            |result| match result {
                Ok(()) => Message::LinkOpened,
                Err(error) => Message::LinkOpenFailed(error),
            },
        )
    }

    fn open_anchor_link(&mut self, anchor: &str) -> Task<Message> {
        let Some(block_index) = self.anchor_block_index(anchor) else {
            self.status = Status::Error(format!("heading anchor not found: #{anchor}"));
            return Task::none();
        };

        self.active_toc_block_index = Some(block_index);
        self.scroll_to_toc_block(block_index)
    }

    fn anchor_block_index(&self, anchor: &str) -> Option<usize> {
        self.documents
            .active()?
            .parsed()
            .toc()
            .into_iter()
            .find_map(|item| (item.slug == anchor).then_some(item.block_index))
    }

    fn update_search_query(&mut self, query: String) -> Task<Message> {
        self.search_query = query;
        self.refresh_search_matches();

        if self.search_matches.is_empty() {
            Task::none()
        } else {
            self.search_selected_index = Some(0);
            self.scroll_to_selected_search_match()
        }
    }

    fn refresh_search_matches(&mut self) {
        self.search_matches = self
            .documents
            .active()
            .map_or_else(Vec::new, |document| document.search(&self.search_query));
        self.search_selected_index =
            clamp_search_selection(self.search_selected_index, self.search_matches.len());
    }

    fn select_next_search_match(&mut self) -> Task<Message> {
        if self.search_matches.is_empty() {
            return Task::none();
        }

        let next = self
            .search_selected_index
            .map_or(0, |index| (index + 1) % self.search_matches.len());
        self.search_selected_index = Some(next);
        self.scroll_to_selected_search_match()
    }

    fn select_previous_search_match(&mut self) -> Task<Message> {
        if self.search_matches.is_empty() {
            return Task::none();
        }

        let previous = self.search_selected_index.map_or(0, |index| {
            if index == 0 {
                self.search_matches.len() - 1
            } else {
                index - 1
            }
        });
        self.search_selected_index = Some(previous);
        self.scroll_to_selected_search_match()
    }

    fn scroll_to_selected_search_match(&self) -> Task<Message> {
        let Some(index) = self.search_selected_index else {
            return Task::none();
        };
        let Some(document) = self.documents.active() else {
            return Task::none();
        };
        let Some(search_match) = self.search_matches.get(index) else {
            return Task::none();
        };

        operation::snap_to(
            reader::ACTIVE_READER_SCROLLABLE_ID,
            RelativeOffset {
                x: 0.0,
                y: search_scroll_progress(document, search_match.line_index),
            },
        )
    }

    fn search_summary(&self) -> String {
        if self.search_query.trim().is_empty() {
            return "0/0".to_owned();
        }

        self.search_selected_index.map_or_else(
            || "0/0".to_owned(),
            |index| format!("{}/{}", index + 1, self.search_matches.len()),
        )
    }

    fn active_search_query(&self) -> Option<&str> {
        let query = self.search_query.trim();
        (!query.is_empty()).then_some(query)
    }
}

fn first_toc_block_index(document: &paperview_core::parser::ParsedDocument) -> Option<usize> {
    document.toc().first().map(|item| item.block_index)
}

fn clamp_search_selection(selection: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(selection.unwrap_or(0).min(len - 1))
    }
}

fn search_scroll_progress(document: &Document, line_index: usize) -> f32 {
    let line_count = document.source().lines().count();

    if line_count <= 1 {
        0.0
    } else {
        (line_index as f32 / (line_count - 1) as f32).clamp(0.0, 1.0)
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
        Some(document) if state.is_zen => reader::view_with_search(
            document,
            Some(Message::ReaderScrolled),
            Message::OpenLink,
            state.active_search_query(),
        ),
        Some(document) => {
            let reader = if let Some(secondary) = state.split_document() {
                let (primary_width, secondary_width) = state.split_widths();

                row![
                    container(reader::view_with_search(
                        document,
                        Some(Message::ReaderScrolled),
                        Message::OpenLink,
                        state.active_search_query()
                    ))
                    .width(Length::FillPortion(primary_width)),
                    container(reader::view(secondary, Message::OpenLink))
                        .width(Length::FillPortion(secondary_width))
                ]
                .spacing(1)
                .into()
            } else {
                reader::view_with_search(
                    document,
                    Some(Message::ReaderScrolled),
                    Message::OpenLink,
                    state.active_search_query(),
                )
            };

            row![
                history::view(&state.history),
                reader,
                navigation::view(
                    document.parsed(),
                    state.active_toc_block_index,
                    Message::TocSelected
                )
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

fn resolve_link_target(target: &str, active_path: Option<&PathBuf>) -> String {
    if is_absolute_link_target(target) {
        return target.to_owned();
    }

    let Some(parent) = active_path.and_then(|path| path.parent()) else {
        return target.to_owned();
    };

    parent.join(target).to_string_lossy().into_owned()
}

fn is_absolute_link_target(target: &str) -> bool {
    target.starts_with('#')
        || target.contains("://")
        || target.starts_with("mailto:")
        || target.starts_with("tel:")
        || PathBuf::from(target).is_absolute()
}

fn open_link_target(target: String) -> Result<(), String> {
    if target.trim().is_empty() {
        return Err("cannot open an empty link".to_owned());
    }

    let status = platform_open_command(&target)
        .status()
        .map_err(|error| format!("failed to open {target}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to open {target}: opener exited with {status}"
        ))
    }
}

#[cfg(target_os = "macos")]
fn platform_open_command(target: &str) -> Command {
    let mut command = Command::new("open");
    command.arg(target);
    command
}

#[cfg(target_os = "windows")]
fn platform_open_command(target: &str) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", "", target]);
    command
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_open_command(target: &str) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(target);
    command
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
    let search_controls = search_controls(state);

    container(
        row![
            column![
                text("PaperView").size(18).color(theme::SHELL_TEXT),
                text(subtitle).size(12).color(theme::SHELL_TEXT_MUTED)
            ]
            .spacing(4)
            .width(Fill),
            search_controls,
            split_button
        ]
        .spacing(12)
        .height(64),
    )
    .padding([14, 18])
    .width(Fill)
    .style(|_| theme::header_container())
    .into()
}

fn search_controls(state: &PaperView) -> Element<'_, Message> {
    let can_search = state.documents.active().is_some();
    let has_matches = !state.search_matches.is_empty();
    let mut input = text_input("Search", &state.search_query)
        .padding([7, 10])
        .size(13)
        .width(220)
        .style(theme::search_input);

    if can_search {
        input = input
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::SearchNext);
    }

    let previous = button(text("<").size(13))
        .padding([7, 10])
        .style(move |_, status| theme::header_action_button(false, status));
    let previous = if has_matches {
        previous.on_press(Message::SearchPrevious)
    } else {
        previous
    };

    let next = button(text(">").size(13))
        .padding([7, 10])
        .style(move |_, status| theme::header_action_button(false, status));
    let next = if has_matches {
        next.on_press(Message::SearchNext)
    } else {
        next
    };

    row![
        input,
        text(state.search_summary())
            .size(12)
            .color(theme::SHELL_TEXT_MUTED),
        previous,
        next
    ]
    .spacing(6)
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
    use paperview_core::{Document, HistoryStore, parser::parse_markdown};

    use super::{
        Message, PaperView, SplitResize, open_link_target, reader, resolve_link_target,
        runtime_event, search_scroll_progress, title, update,
    };

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

        apply(&mut state, Message::OpenHistory(path.clone()));

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
        apply(&mut state, Message::FileChanged(path.clone()));

        assert_eq!(state.documents.active().map(Document::title), Some("After"));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn opening_document_selects_first_toc_item() {
        let path = temp_doc("toc-open.md", "# First\n\nText.\n\n## Second\n\nMore.");
        let state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("toc-open.toml"));

        assert_eq!(state.active_toc_block_index, Some(0));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn reader_scroll_updates_active_toc_item() {
        let path = temp_doc(
            "toc-scroll.md",
            "# First\n\nText.\n\n## Second\n\nMore.\n\n## Third\n\nEnd.",
        );
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("toc-scroll.toml"));

        apply(&mut state, Message::ReaderScrolled(0.6));

        assert_eq!(state.active_toc_block_index, Some(2));

        apply(&mut state, Message::ReaderScrolled(0.9));

        assert_eq!(state.active_toc_block_index, Some(4));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn active_toc_mapping_is_bounded_and_ignores_empty_toc() {
        let parsed = parse_markdown("# First\n\nText.\n\n## Second\n\nMore.");

        assert_eq!(reader::active_heading_for_scroll(&parsed, -1.0), Some(0));
        assert_eq!(
            reader::active_heading_for_scroll(&parsed, f32::NAN),
            Some(0)
        );
        assert_eq!(reader::active_heading_for_scroll(&parsed, 1.5), Some(2));
        assert_eq!(
            reader::active_heading_for_scroll(&parse_markdown("No headings."), 0.5),
            None
        );
    }

    #[test]
    fn toc_selection_updates_active_toc_item() {
        let path = temp_doc("toc-select.md", "# First\n\nText.\n\n## Second\n\nMore.");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("toc-select.toml"));

        apply(&mut state, Message::TocSelected(2));

        assert_eq!(state.active_toc_block_index, Some(2));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn toc_block_scroll_progress_is_bounded() {
        let parsed = parse_markdown("# First\n\nText.\n\n## Second\n\nMore.");

        assert_eq!(reader::heading_scroll_progress(&parsed, 0), 0.0);
        assert_eq!(reader::heading_scroll_progress(&parsed, usize::MAX), 1.0);
        assert_eq!(
            reader::heading_scroll_progress(&parse_markdown("# Only"), 0),
            0.0
        );
    }

    #[test]
    fn search_query_updates_gui_matches() {
        let path = temp_doc(
            "gui-search.md",
            "# First\n\nNeedle here.\n\nAnother needle.",
        );
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("search.toml"));

        apply(&mut state, Message::SearchQueryChanged("needle".to_owned()));

        assert_eq!(state.search_matches.len(), 2);
        assert_eq!(state.search_selected_index, Some(0));
        assert_eq!(state.search_summary(), "1/2");

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn search_navigation_wraps_gui_matches() {
        let path = temp_doc(
            "gui-search-wrap.md",
            "Needle one.\n\nMiddle.\n\nNeedle two.",
        );
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("search-wrap.toml"),
        );

        apply(&mut state, Message::SearchQueryChanged("needle".to_owned()));
        apply(&mut state, Message::SearchNext);
        assert_eq!(state.search_selected_index, Some(1));

        apply(&mut state, Message::SearchNext);
        assert_eq!(state.search_selected_index, Some(0));

        apply(&mut state, Message::SearchPrevious);
        assert_eq!(state.search_selected_index, Some(1));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn search_matches_refresh_after_active_tab_changes() {
        let first = temp_doc("gui-search-first.md", "# First\n\nNeedle.");
        let second = temp_doc("gui-search-second.md", "# Second\n\nNo match.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("search-tabs.toml"),
        );

        apply(&mut state, Message::SearchQueryChanged("needle".to_owned()));
        assert_eq!(state.search_matches.len(), 1);

        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        assert!(state.search_matches.is_empty());
        assert_eq!(state.search_selected_index, None);

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn search_scroll_progress_uses_source_line_position() {
        let document = Document::from_source("one\nneedle\ntwo\nthree");

        assert_eq!(search_scroll_progress(&document, 0), 0.0);
        assert!(search_scroll_progress(&document, 1) > 0.3);
        assert_eq!(search_scroll_progress(&document, usize::MAX), 1.0);
    }

    #[test]
    fn dropped_file_opens_document() {
        let path = temp_doc("dropped.md", "# Dropped\n\nOpened from drop.");
        let mut state = PaperView::from_args_with_store([], temp_store("drop.toml"));

        apply(&mut state, Message::OpenDroppedFiles(vec![path.clone()]));

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

        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

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

        apply(
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

        apply(
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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        apply(&mut state, Message::SelectTab(0));

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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::OpenDroppedFiles(vec![third.clone()]));
        apply(&mut state, Message::SelectTab(1));

        apply(&mut state, Message::CloseTab(1));

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

        apply(&mut state, Message::CloseTab(0));

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

        apply(&mut state, Message::ToggleSplit);

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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        apply(&mut state, Message::ToggleSplit);

        assert_eq!(state.documents.active_index(), Some(1));
        assert_eq!(state.split_document_index, Some(0));
        assert_eq!(state.split_document().map(Document::title), Some("First"));

        apply(&mut state, Message::ToggleSplit);

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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));

        assert_eq!(state.split_widths(), (60, 40));

        apply(&mut state, Message::ResizeSplit(SplitResize::ShrinkPrimary));
        apply(&mut state, Message::ResizeSplit(SplitResize::ShrinkPrimary));

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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        apply(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));
        assert_eq!(state.split_widths(), (50, 50));

        apply(&mut state, Message::ToggleSplit);
        for _ in 0..8 {
            apply(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));
        }
        assert_eq!(state.split_widths(), (70, 30));

        for _ in 0..8 {
            apply(&mut state, Message::ResizeSplit(SplitResize::ShrinkPrimary));
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
        apply(
            &mut state,
            Message::OpenDroppedFiles(vec![second.clone(), third.clone()]),
        );
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::SelectTab(0));

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
        apply(
            &mut state,
            Message::OpenDroppedFiles(vec![second.clone(), third.clone()]),
        );
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::SelectSplitTab(1));

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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::SelectSplitTab(1));

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
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::CloseTab(0));

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

        apply(&mut state, Message::FileHovered(hover_path.clone()));
        assert!(state.is_drag_hovered);
        assert!(matches!(state.status, super::Status::Hovering(_)));

        apply(&mut state, Message::FilesHoveredLeft);

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

        apply(&mut state, Message::ToggleZen);
        assert!(state.is_zen);

        apply(&mut state, Message::ToggleZen);
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

    #[test]
    fn relative_links_resolve_against_active_document_parent() {
        let path = temp_doc("links-source.md", "# Links\n\nSee [docs](docs/guide.md).");
        let expected = path
            .parent()
            .expect("temp doc parent")
            .join("docs/guide.md");

        assert_eq!(
            resolve_link_target("docs/guide.md", Some(&path)),
            expected.to_string_lossy()
        );
        assert_eq!(
            resolve_link_target("https://example.com", Some(&path)),
            "https://example.com"
        );
        assert_eq!(resolve_link_target("#section", Some(&path)), "#section");

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn empty_links_are_rejected_before_platform_open() {
        assert_eq!(
            open_link_target(String::new()).expect_err("reject empty link"),
            "cannot open an empty link"
        );
    }

    #[test]
    fn anchor_links_select_matching_heading() {
        let path = temp_doc(
            "anchor-link.md",
            "# Intro\n\n[Jump](#details)\n\n## Details",
        );
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("anchor.toml"));

        apply(&mut state, Message::OpenLink("#details".to_owned()));

        assert_eq!(state.active_toc_block_index, Some(2));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn anchor_links_use_duplicate_heading_slugs() {
        let path = temp_doc(
            "anchor-duplicates.md",
            "# Intro\n\n## Details\n\n## Details\n\n[Jump](#details-2)",
        );
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("anchor-duplicates.toml"),
        );

        apply(&mut state, Message::OpenLink("#details-2".to_owned()));

        assert_eq!(state.active_toc_block_index, Some(2));

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn missing_anchor_links_set_error_status() {
        let path = temp_doc("anchor-missing.md", "# Intro\n\n[Missing](#missing)");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("anchor-missing.toml"),
        );

        apply(&mut state, Message::OpenLink("#missing".to_owned()));

        assert!(
            matches!(state.status, super::Status::Error(ref error) if error == "heading anchor not found: #missing")
        );

        fs::remove_file(path).expect("remove test document");
    }

    fn temp_store(name: &str) -> HistoryStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        HistoryStore::new(std::env::temp_dir().join(format!("paperview-gui-{nanos}-{name}")))
    }

    fn apply(state: &mut PaperView, message: Message) {
        let _ = update(state, message);
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
