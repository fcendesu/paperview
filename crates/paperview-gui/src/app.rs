use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
};

use iced::{
    Element, Event, Fill, Length, Subscription, Task,
    event::{self, Status as EventStatus},
    futures::{SinkExt, StreamExt, stream::BoxStream},
    keyboard, mouse,
    widget::{
        button, column, container, mouse_area,
        operation::{self, RelativeOffset},
        responsive, row, scrollable, text, text_editor, text_input,
    },
    window,
};
use paperview_core::{
    Config, ConfigStore, Document, EditSession, History, HistoryStore, OpenDocuments,
    PresentationDeck, SearchMatch, SplitResize, SplitViewState, SupportedFileType, WatchEvent,
    WorkspaceSearchMatch, ZenModeState, parser::Block, presentation_deck, search_workspace,
    toggle_task_line_source, watch_file,
};

use crate::{
    editor_highlight::{MarkdownHighlighter, markdown_highlight_format},
    navigation, reader, theme,
};

const SPLIT_DIVIDER_HIT_ZONE: f32 = 16.0;
const MAX_REMOTE_IMAGE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct PaperView {
    config: Config,
    config_store: ConfigStore,
    documents: OpenDocuments,
    history: History,
    history_store: HistoryStore,
    status: Status,
    is_drag_hovered: bool,
    zen_mode: ZenModeState,
    split_view: SplitViewState,
    split_drag_cursor: Option<SplitDragCursor>,
    is_split_dragging: bool,
    active_toc_block_index: Option<usize>,
    search_query: String,
    search_matches: Vec<SearchMatch>,
    search_selected_index: Option<usize>,
    workspace_search_query: String,
    workspace_search_matches: Vec<WorkspaceSearchMatch>,
    is_workspace_searching: bool,
    workspace_search_error: Option<String>,
    remote_images: HashMap<String, reader::RemoteImage>,
    edit_session: Option<EditSession>,
    edit_content: text_editor::Content,
    edit_preview: Option<Document>,
    presentation_deck: Option<PresentationDeck>,
    presentation_slide_index: usize,
    presentation_document: Option<Document>,
}

#[derive(Debug, Clone)]
enum Status {
    Empty,
    Loaded(PathBuf),
    CompilingTex(PathBuf),
    CompiledTex { source: PathBuf, output: PathBuf },
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
    SplitDragMoved {
        x: f32,
        width: f32,
    },
    SplitDragStarted,
    SplitDragEnded,
    ReaderScrolled(f32),
    OpenLink(String),
    LinkOpened,
    LinkOpenFailed(String),
    TexCompileFinished {
        source: PathBuf,
        result: Result<PathBuf, String>,
    },
    RemoteImageLoaded {
        url: String,
        result: Result<Vec<u8>, String>,
    },
    TocSelected(usize),
    SearchQueryChanged(String),
    SearchNext,
    SearchPrevious,
    WorkspaceSearchQueryChanged(String),
    SubmitWorkspaceSearch,
    WorkspaceSearchFinished(Result<Vec<WorkspaceSearchMatch>, String>),
    OpenWorkspaceSearchResult(usize),
    ToggleEdit,
    EditSource(text_editor::Action),
    SaveEdit,
    TogglePresentation,
    PresentationNext,
    PresentationPrevious,
    PresentationFirst,
    PresentationLast,
    PresentationExit,
    ToggleTask(usize),
    SelectSplitTab(usize),
    SelectTab(usize),
    CloseTab(usize),
}

#[derive(Debug, Clone, Copy)]
struct SplitDragCursor {
    x: f32,
    width: f32,
}

#[derive(Debug, Clone, Hash)]
struct ActiveWatchPath(PathBuf);

#[derive(Debug, Clone, Hash)]
struct RemoteImageUrl(String);

impl PaperView {
    #[must_use]
    pub fn from_args(args: impl IntoIterator<Item = OsString>) -> Self {
        Self::from_args_with_stores(args, HistoryStore::default(), ConfigStore::default())
    }

    #[must_use]
    #[cfg(test)]
    fn from_args_with_store(
        args: impl IntoIterator<Item = OsString>,
        history_store: HistoryStore,
    ) -> Self {
        let config_store = test_config_store(&history_store);
        Self::from_args_with_stores(args, history_store, config_store)
    }

    #[must_use]
    fn from_args_with_stores(
        args: impl IntoIterator<Item = OsString>,
        history_store: HistoryStore,
        config_store: ConfigStore,
    ) -> Self {
        let args = args.into_iter().collect::<Vec<_>>();
        let config = load_config(&config_store);
        let zen_mode = ZenModeState::new(config.zen_mode);
        let split_primary_width = config.split_primary_width.clamp(
            SplitViewState::MIN_PRIMARY_WIDTH,
            SplitViewState::MAX_PRIMARY_WIDTH,
        );

        match args.as_slice() {
            [] => {
                let history = load_history(&history_store);

                Self::new_empty(
                    config,
                    config_store,
                    history,
                    history_store,
                    zen_mode,
                    split_primary_width,
                    Status::Empty,
                )
            }
            [path] => {
                let path = PathBuf::from(path);
                let mut history = load_history(&history_store);

                if is_tex_document_path(&path) {
                    let mut state = Self::new_empty(
                        config,
                        config_store,
                        history,
                        history_store,
                        zen_mode,
                        split_primary_width,
                        Status::Empty,
                    );
                    state.open_tex_path_sync(path, true);
                    return state;
                }

                match Document::open(&path) {
                    Ok(document) => {
                        let active_toc_block_index = first_toc_block_index(document.parsed());
                        let remote_images = remote_image_placeholders(Some(&document));

                        history.record_document(&document);
                        save_history(&history_store, &history);

                        Self {
                            config: config.clone(),
                            config_store: config_store.clone(),
                            documents: OpenDocuments::from_document(document),
                            history,
                            history_store,
                            status: Status::Loaded(path),
                            is_drag_hovered: false,
                            zen_mode,
                            split_view: SplitViewState::new(split_primary_width),
                            split_drag_cursor: None,
                            is_split_dragging: false,
                            active_toc_block_index,
                            search_query: String::new(),
                            search_matches: Vec::new(),
                            search_selected_index: None,
                            workspace_search_query: String::new(),
                            workspace_search_matches: Vec::new(),
                            is_workspace_searching: false,
                            workspace_search_error: None,
                            remote_images,
                            edit_session: None,
                            edit_content: text_editor::Content::new(),
                            edit_preview: None,
                            presentation_deck: None,
                            presentation_slide_index: 0,
                            presentation_document: None,
                        }
                    }
                    Err(error) => Self::new_empty(
                        config,
                        config_store,
                        history,
                        history_store,
                        zen_mode,
                        split_primary_width,
                        Status::Error(error.to_string()),
                    ),
                }
            }
            _ => {
                let history = load_history(&history_store);

                Self::new_empty(
                    config,
                    config_store,
                    history,
                    history_store,
                    zen_mode,
                    split_primary_width,
                    Status::Error("usage: paperview-gui [file]".to_owned()),
                )
            }
        }
    }

    fn new_empty(
        config: Config,
        config_store: ConfigStore,
        history: History,
        history_store: HistoryStore,
        zen_mode: ZenModeState,
        split_primary_width: u16,
        status: Status,
    ) -> Self {
        Self {
            config,
            config_store,
            documents: OpenDocuments::new(),
            history,
            history_store,
            status,
            is_drag_hovered: false,
            zen_mode,
            split_view: SplitViewState::new(split_primary_width),
            split_drag_cursor: None,
            is_split_dragging: false,
            active_toc_block_index: None,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_selected_index: None,
            workspace_search_query: String::new(),
            workspace_search_matches: Vec::new(),
            is_workspace_searching: false,
            workspace_search_error: None,
            remote_images: HashMap::new(),
            edit_session: None,
            edit_content: text_editor::Content::new(),
            edit_preview: None,
            presentation_deck: None,
            presentation_slide_index: 0,
            presentation_document: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StartupProbe {
    pub(crate) document_count: usize,
    pub(crate) history_entries: usize,
    pub(crate) active_toc_items: usize,
    pub(crate) remote_image_placeholders: usize,
    pub(crate) status: &'static str,
}

pub(crate) fn probe_startup(args: impl IntoIterator<Item = OsString>) -> StartupProbe {
    let state = PaperView::from_args(args);
    let active_toc_items = state
        .documents
        .active()
        .map_or(0, |document| document.parsed().toc().len());

    StartupProbe {
        document_count: state.documents.len(),
        history_entries: state.history.entries().len(),
        active_toc_items,
        remote_image_placeholders: state.remote_images.len(),
        status: state.status.label(),
    }
}

impl Status {
    fn label(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Loaded(_) => "loaded",
            Self::CompilingTex(_) => "compiling_tex",
            Self::CompiledTex { .. } => "compiled_tex",
            Self::Hovering(_) => "hovering",
            Self::Error(_) => "error",
        }
    }
}

pub fn update(state: &mut PaperView, message: Message) -> Task<Message> {
    match message {
        Message::OpenHistory(path) => {
            return state.open_path(path);
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
            return state.open_dropped_files(paths);
        }
        Message::ToggleZen => {
            state.zen_mode.toggle();
            state.save_config();
        }
        Message::ToggleSplit => state.toggle_split(),
        Message::ResizeSplit(direction) => state.resize_split(direction),
        Message::SplitDragMoved { x, width } => state.move_split_drag(x, width),
        Message::SplitDragStarted => state.start_split_drag(),
        Message::SplitDragEnded => state.end_split_drag(),
        Message::ReaderScrolled(progress) => return state.sync_active_toc_to_scroll(progress),
        Message::OpenLink(target) => return state.open_link(target),
        Message::LinkOpened => {}
        Message::LinkOpenFailed(error) => {
            state.status = Status::Error(error);
        }
        Message::TexCompileFinished { source, result } => {
            state.finish_tex_compile(source, result);
        }
        Message::RemoteImageLoaded { url, result } => {
            let image =
                result.map_or_else(reader::RemoteImage::Failed, reader::RemoteImage::Loaded);
            state.remote_images.insert(url, image);
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
        Message::WorkspaceSearchQueryChanged(query) => state.update_workspace_search_query(query),
        Message::SubmitWorkspaceSearch => return state.submit_workspace_search(),
        Message::WorkspaceSearchFinished(result) => state.finish_workspace_search(result),
        Message::OpenWorkspaceSearchResult(index) => {
            return state.open_workspace_search_result(index);
        }
        Message::ToggleEdit => state.toggle_edit(),
        Message::EditSource(action) => state.edit_source(action),
        Message::SaveEdit => state.save_edit(),
        Message::TogglePresentation => state.toggle_presentation(),
        Message::PresentationNext => state.select_next_presentation_slide(),
        Message::PresentationPrevious => state.select_previous_presentation_slide(),
        Message::PresentationFirst => state.select_first_presentation_slide(),
        Message::PresentationLast => state.select_last_presentation_slide(),
        Message::PresentationExit => state.exit_presentation(),
        Message::ToggleTask(line_index) => state.toggle_task(line_index),
        Message::SelectSplitTab(index) => state.select_split_tab(index),
        Message::SelectTab(index) => state.select_tab(index),
        Message::CloseTab(index) => state.close_tab(index),
    }

    Task::none()
}

impl PaperView {
    fn open_path(&mut self, path: PathBuf) -> Task<Message> {
        if is_tex_document_path(&path) {
            return self.open_tex_path(path, true);
        }

        self.open_reader_path(path);
        Task::none()
    }

    fn open_reader_path(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(document) => {
                self.history.record_document(&document);
                save_history(&self.history_store, &self.history);
                self.documents.open_or_activate(document);
                self.status = Status::Loaded(path);
                self.end_editing();
                self.end_presentation();
                self.ensure_split_target();
                self.sync_active_toc_to_top();
                self.refresh_search_matches();
                self.track_remote_images();
            }
            Err(error) => {
                self.status = Status::Error(error.to_string());
            }
        }
    }

    fn open_tex_path(&mut self, path: PathBuf, open_after_compile: bool) -> Task<Message> {
        self.status = Status::CompilingTex(path.clone());
        self.end_editing();
        self.end_presentation();

        let config = self.config.clone();
        let source = path.clone();

        Task::perform(
            async move { compile_tex_for_gui(&path, &config, open_after_compile) },
            move |result| Message::TexCompileFinished {
                source: source.clone(),
                result,
            },
        )
    }

    fn open_tex_path_sync(&mut self, path: PathBuf, open_after_compile: bool) {
        self.status = Status::CompilingTex(path.clone());
        self.end_editing();
        self.end_presentation();
        let result = compile_tex_for_gui(&path, &self.config, open_after_compile);
        self.finish_tex_compile(path, result);
    }

    fn finish_tex_compile(&mut self, source: PathBuf, result: Result<PathBuf, String>) {
        match result {
            Ok(output) => {
                self.status = Status::CompiledTex { source, output };
            }
            Err(error) => {
                self.status = Status::Error(error);
            }
        }
    }

    fn open_dropped_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> Task<Message> {
        let mut last_error = None;
        let mut tasks = Vec::new();

        for path in paths {
            if is_tex_document_path(&path) {
                tasks.push(self.open_tex_path(path, true));
            } else {
                self.open_reader_path(path);
                if let Status::Error(error) = &self.status {
                    last_error = Some(error.clone());
                }
            }
        }

        if let Some(error) = last_error {
            self.status = Status::Error(error);
        }

        Task::batch(tasks)
    }

    fn reload_path(&mut self, path: PathBuf) {
        if self.is_active_path(&path) {
            match Document::open(&path) {
                Ok(document) => {
                    self.history.record_document(&document);
                    save_history(&self.history_store, &self.history);
                    self.documents.replace_active(document);
                    self.status = Status::Loaded(path);
                    self.end_editing();
                    self.end_presentation();
                    self.sync_active_toc_to_top();
                    self.refresh_search_matches();
                    self.track_remote_images();
                }
                Err(error) => {
                    self.status = Status::Error(error.to_string());
                }
            }
            return;
        }

        let Some(split_index) = self.split_index_for_path(&path) else {
            return;
        };

        match Document::open(&path) {
            Ok(document) => {
                self.documents.replace_at(split_index, document);
                self.status = Status::Loaded(path);
                self.track_remote_images();
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

    fn split_path(&self) -> Option<PathBuf> {
        self.split_document()
            .and_then(Document::path)
            .map(PathBuf::from)
    }

    fn split_index_for_path(&self, path: &PathBuf) -> Option<usize> {
        let split_index = self.split_view.secondary_index()?;
        self.documents.iter().find_map(|(index, document)| {
            (index == split_index && document.path() == Some(path)).then_some(index)
        })
    }

    fn select_tab(&mut self, index: usize) {
        self.documents.select(index);
        self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
        self.end_editing();
        self.end_presentation();
        self.ensure_split_target();
        self.sync_active_toc_to_top();
        self.refresh_search_matches();
        self.track_remote_images();
    }

    fn close_tab(&mut self, index: usize) {
        self.documents.close(index);
        self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
        self.end_editing();
        self.end_presentation();
        self.ensure_split_target();
        self.sync_active_toc_to_top();
        self.refresh_search_matches();
        self.track_remote_images();
    }

    fn select_split_tab(&mut self, index: usize) {
        let previous = self.split_view.secondary_index();
        self.split_view.select_secondary(
            index,
            self.documents.active_index(),
            self.documents.len(),
        );
        if self.split_view.secondary_index() != previous {
            self.track_remote_images();
        }
    }

    fn toggle_edit(&mut self) {
        if self.edit_session.is_some() {
            self.end_editing();
            self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
            return;
        }

        let Some(document) = self.documents.active() else {
            self.status = Status::Error("open a document before editing".to_owned());
            return;
        };

        let session = EditSession::from_document(document);
        self.end_presentation();
        self.edit_content = text_editor::Content::with_text(session.buffer());
        self.edit_preview = Some(session.preview_document());
        self.edit_session = Some(session);
    }

    fn toggle_presentation(&mut self) {
        if self.presentation_deck.is_some() {
            self.end_presentation();
            self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
            return;
        }

        let Some(document) = self.documents.active() else {
            self.status = Status::Error("open a document before presenting".to_owned());
            return;
        };

        let deck = presentation_deck(document.source());
        if deck.is_empty() {
            self.status = Status::Error("presentation has no slides".to_owned());
            return;
        }

        self.end_editing();
        self.presentation_deck = Some(deck);
        self.presentation_slide_index = 0;
        self.refresh_presentation_document();
    }

    fn select_next_presentation_slide(&mut self) {
        let Some(deck) = &self.presentation_deck else {
            return;
        };

        if self.presentation_slide_index + 1 < deck.len() {
            self.presentation_slide_index += 1;
            self.refresh_presentation_document();
        }
    }

    fn select_previous_presentation_slide(&mut self) {
        if self.presentation_slide_index > 0 {
            self.presentation_slide_index -= 1;
            self.refresh_presentation_document();
        }
    }

    fn select_first_presentation_slide(&mut self) {
        if self.presentation_deck.is_some() && self.presentation_slide_index > 0 {
            self.presentation_slide_index = 0;
            self.refresh_presentation_document();
        }
    }

    fn select_last_presentation_slide(&mut self) {
        let Some(deck) = &self.presentation_deck else {
            return;
        };
        let last_index = deck.len().saturating_sub(1);
        if self.presentation_slide_index != last_index {
            self.presentation_slide_index = last_index;
            self.refresh_presentation_document();
        }
    }

    fn exit_presentation(&mut self) {
        if self.presentation_deck.is_some() {
            self.end_presentation();
            self.status = self.active_path().map_or(Status::Empty, Status::Loaded);
        }
    }

    fn refresh_presentation_document(&mut self) {
        let active_path = self.active_path();
        self.presentation_document = self
            .presentation_deck
            .as_ref()
            .and_then(|deck| deck.slides().get(self.presentation_slide_index))
            .map(|slide| {
                let document = Document::from_source(slide.source());
                match active_path.clone() {
                    Some(path) => document.with_path(path),
                    None => document,
                }
            });
        self.active_toc_block_index = self
            .presentation_document
            .as_ref()
            .and_then(|document| first_toc_block_index(document.parsed()));
    }

    fn end_presentation(&mut self) {
        self.presentation_deck = None;
        self.presentation_slide_index = 0;
        self.presentation_document = None;
        self.active_toc_block_index = self
            .documents
            .active()
            .and_then(|document| first_toc_block_index(document.parsed()));
    }

    fn edit_source(&mut self, action: text_editor::Action) {
        self.edit_content.perform(action);
        let Some(session) = &mut self.edit_session else {
            return;
        };

        session.replace_buffer(self.edit_content.text());
        self.edit_preview = Some(session.preview_document());
    }

    fn save_edit(&mut self) {
        let Some(session) = &mut self.edit_session else {
            self.status = Status::Error("enter Editing Mode before saving".to_owned());
            return;
        };

        match session.save() {
            Ok(document) => {
                let path = document.path().cloned();
                self.documents.replace_active(document);
                self.edit_content = text_editor::Content::with_text(session.buffer());
                self.edit_preview = Some(session.preview_document());
                self.status = path.map_or(Status::Empty, Status::Loaded);
                self.sync_active_toc_to_top();
                self.refresh_search_matches();
                self.track_remote_images();
            }
            Err(error) => {
                self.status = Status::Error(error.to_string());
            }
        }
    }

    fn end_editing(&mut self) {
        self.edit_session = None;
        self.edit_content = text_editor::Content::new();
        self.edit_preview = None;
    }

    fn toggle_task(&mut self, line_index: usize) {
        let Some(path) = self.active_path() else {
            self.status = Status::Error("task toggles require a file-backed document".to_owned());
            return;
        };
        let Some(document) = self.documents.active() else {
            return;
        };
        let Some(updated_source) = toggle_task_line_source(document.source(), line_index) else {
            self.status = Status::Error("task checkbox source line was not found".to_owned());
            return;
        };

        if let Err(error) = fs::write(&path, updated_source) {
            self.status = Status::Error(format!("failed to update {}: {error}", path.display()));
            return;
        }

        self.reload_path(path);
    }

    fn toggle_split(&mut self) {
        self.split_view
            .toggle(self.documents.active_index(), self.documents.len());
        self.end_split_drag();
        self.track_remote_images();
    }

    fn resize_split(&mut self, direction: SplitResize) {
        if self.split_view.resize(direction) {
            self.save_config();
        }
    }

    fn move_split_drag(&mut self, x: f32, width: f32) {
        self.split_drag_cursor = Some(SplitDragCursor { x, width });

        if self.is_split_dragging {
            self.split_view
                .set_primary_width(split_primary_width_from_cursor(
                    x,
                    width,
                    self.split_view.primary_width(),
                ));
        }
    }

    fn start_split_drag(&mut self) {
        if !self.split_view.is_enabled() {
            return;
        }

        self.is_split_dragging = true;
    }

    fn end_split_drag(&mut self) {
        let was_dragging = self.is_split_dragging;
        self.is_split_dragging = false;
        if was_dragging {
            self.save_config();
        }
    }

    fn is_split_divider_hovered(&self) -> bool {
        self.split_drag_cursor.is_some_and(|cursor| {
            is_split_divider_hit(cursor.x, cursor.width, self.split_view.primary_width())
        })
    }

    fn ensure_split_target(&mut self) {
        self.split_view
            .retarget(self.documents.active_index(), self.documents.len());
    }

    fn split_document(&self) -> Option<&Document> {
        let split_index = self.split_view.secondary_index()?;

        self.documents
            .iter()
            .find_map(|(index, document)| (index == split_index).then_some(document))
    }

    fn split_widths(&self) -> (u16, u16) {
        self.split_view.widths()
    }

    fn track_remote_images(&mut self) {
        for url in visible_remote_image_urls(&self.documents, self.split_view.secondary_index()) {
            self.remote_images
                .entry(url)
                .or_insert(reader::RemoteImage::Loading);
        }
    }

    fn sync_active_toc_to_top(&mut self) {
        self.active_toc_block_index = self
            .documents
            .active()
            .and_then(|document| first_toc_block_index(document.parsed()));
    }

    fn sync_active_toc_to_scroll(&mut self, progress: f32) -> Task<Message> {
        let Some(document) = self.documents.active() else {
            self.active_toc_block_index = None;
            return Task::none();
        };

        self.active_toc_block_index =
            reader::active_heading_for_scroll(document.parsed(), progress);
        self.sync_split_scroll(progress)
    }

    fn sync_split_scroll(&self, progress: f32) -> Task<Message> {
        if self.split_view.secondary_index().is_none() || self.zen_mode.is_enabled() {
            return Task::none();
        }

        operation::snap_to(
            reader::SPLIT_READER_SCROLLABLE_ID,
            RelativeOffset {
                x: 0.0,
                y: normalized_scroll_progress(progress),
            },
        )
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

        if let Some(path) = resolve_local_document_link(&target, self.documents.active()) {
            return self.open_path(path);
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

    fn update_workspace_search_query(&mut self, query: String) {
        self.workspace_search_query = query;
        self.workspace_search_error = None;

        if self.workspace_search_query.trim().is_empty() {
            self.workspace_search_matches.clear();
            self.is_workspace_searching = false;
        }
    }

    fn submit_workspace_search(&mut self) -> Task<Message> {
        let query = self.workspace_search_query.trim().to_owned();
        if query.is_empty() {
            self.workspace_search_matches.clear();
            self.workspace_search_error = None;
            self.is_workspace_searching = false;
            return Task::none();
        }

        self.is_workspace_searching = true;
        self.workspace_search_error = None;

        Task::perform(
            run_workspace_search(query),
            Message::WorkspaceSearchFinished,
        )
    }

    fn finish_workspace_search(&mut self, result: Result<Vec<WorkspaceSearchMatch>, String>) {
        self.is_workspace_searching = false;
        match result {
            Ok(matches) => {
                self.workspace_search_matches = matches;
                self.workspace_search_error = None;
            }
            Err(error) => {
                self.workspace_search_matches.clear();
                self.workspace_search_error = Some(error.clone());
                self.status = Status::Error(error);
            }
        }
    }

    fn open_workspace_search_result(&mut self, index: usize) -> Task<Message> {
        let Some(search_match) = self.workspace_search_matches.get(index).cloned() else {
            return Task::none();
        };
        let path = absolute_workspace_match_path(search_match.path);
        let line_index = search_match.line_number.saturating_sub(1);

        if is_tex_document_path(&path) {
            return self.open_path(path);
        }

        self.open_reader_path(path);
        self.status = self.active_path().map_or(Status::Empty, Status::Loaded);

        operation::snap_to(
            reader::ACTIVE_READER_SCROLLABLE_ID,
            RelativeOffset {
                x: 0.0,
                y: self
                    .documents
                    .active()
                    .map_or(0.0, |document| search_scroll_progress(document, line_index)),
            },
        )
    }

    fn active_search_line(&self) -> Option<&str> {
        self.search_selected_index
            .and_then(|index| self.search_matches.get(index))
            .map(|search_match| search_match.line.as_str())
    }

    fn save_config(&mut self) {
        self.config.zen_mode = self.zen_mode.is_enabled();
        self.config.split_primary_width = self.split_view.primary_width();
        if let Err(error) = self.config_store.save(&self.config) {
            self.status = Status::Error(error.to_string());
        }
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

async fn run_workspace_search(query: String) -> Result<Vec<WorkspaceSearchMatch>, String> {
    let root = std::env::current_dir().map_err(|error| format!("failed to read cwd: {error}"))?;
    search_workspace(&query, root).map_err(|error| error.to_string())
}

fn absolute_workspace_match_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(path)
}

fn split_primary_width_from_cursor(x: f32, width: f32, fallback: u16) -> u16 {
    if width <= 0.0 {
        return fallback.clamp(
            SplitViewState::MIN_PRIMARY_WIDTH,
            SplitViewState::MAX_PRIMARY_WIDTH,
        );
    }

    ((x / width) * 100.0).round().clamp(
        SplitViewState::MIN_PRIMARY_WIDTH as f32,
        SplitViewState::MAX_PRIMARY_WIDTH as f32,
    ) as u16
}

fn is_split_divider_hit(x: f32, width: f32, primary_width: u16) -> bool {
    if width <= 0.0 {
        return false;
    }

    let divider_x = width * f32::from(primary_width) / 100.0;

    (x - divider_x).abs() <= SPLIT_DIVIDER_HIT_ZONE
}

fn remote_image_placeholders(document: Option<&Document>) -> HashMap<String, reader::RemoteImage> {
    let mut images = HashMap::new();

    if let Some(document) = document {
        track_document_remote_images(document, &mut images);
    }

    images
}

fn visible_remote_image_urls(
    documents: &OpenDocuments,
    split_document_index: Option<usize>,
) -> Vec<String> {
    let mut urls = Vec::new();

    if let Some(document) = documents.active() {
        collect_document_remote_image_urls(document, &mut urls);
    }

    if let Some(split_index) = split_document_index
        && let Some((_, document)) = documents.iter().find(|(index, _)| *index == split_index)
    {
        collect_document_remote_image_urls(document, &mut urls);
    }

    urls
}

fn track_document_remote_images(
    document: &Document,
    images: &mut HashMap<String, reader::RemoteImage>,
) {
    for url in document_remote_image_urls(document) {
        images.insert(url, reader::RemoteImage::Loading);
    }
}

fn collect_document_remote_image_urls(document: &Document, urls: &mut Vec<String>) {
    for url in document_remote_image_urls(document) {
        if !urls.contains(&url) {
            urls.push(url);
        }
    }
}

fn document_remote_image_urls(document: &Document) -> impl Iterator<Item = String> + '_ {
    document
        .parsed()
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Image { url, .. } if reader::is_fetchable_remote_image_url(url) => {
                Some(url.to_owned())
            }
            _ => None,
        })
}

fn search_scroll_progress(document: &Document, line_index: usize) -> f32 {
    let line_count = document.source().lines().count();

    if line_count <= 1 {
        0.0
    } else {
        (line_index as f32 / (line_count - 1) as f32).clamp(0.0, 1.0)
    }
}

fn normalized_scroll_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn subscription(state: &PaperView) -> Subscription<Message> {
    let runtime_events = event::listen_with(runtime_event);
    let active_watch = state.active_path().map_or_else(Subscription::none, |path| {
        Subscription::run_with(ActiveWatchPath(path), watch_active_document)
    });
    let split_watch = state.split_path().map_or_else(Subscription::none, |path| {
        Subscription::run_with(ActiveWatchPath(path), watch_active_document)
    });
    let remote_images = state
        .remote_images
        .iter()
        .filter(|(_, image)| matches!(image, reader::RemoteImage::Loading))
        .map(|(url, _)| Subscription::run_with(RemoteImageUrl(url.clone()), watch_remote_image));

    Subscription::batch(
        [runtime_events, active_watch, split_watch]
            .into_iter()
            .chain(remote_images),
    )
}

fn runtime_event(event: Event, _status: EventStatus, _window: window::Id) -> Option<Message> {
    match event {
        Event::Window(window::Event::FileHovered(path)) => Some(Message::FileHovered(path)),
        Event::Window(window::Event::FilesHoveredLeft) => Some(Message::FilesHoveredLeft),
        Event::Window(window::Event::FileDropped(path)) => {
            Some(Message::OpenDroppedFiles(vec![path]))
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            Some(Message::SplitDragEnded)
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
                .is_some_and(|character| character.eq_ignore_ascii_case(&'e')) =>
        {
            Some(Message::ToggleEdit)
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
                .is_some_and(|character| character.eq_ignore_ascii_case(&'p')) =>
        {
            Some(Message::TogglePresentation)
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
                .is_some_and(|character| character.eq_ignore_ascii_case(&'s')) =>
        {
            Some(Message::SaveEdit)
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
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && matches!(
                key,
                keyboard::Key::Named(
                    keyboard::key::Named::ArrowRight | keyboard::key::Named::Space
                )
            ) =>
        {
            Some(Message::PresentationNext)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && key
                .to_latin(physical_key)
                .is_some_and(|character| character.eq_ignore_ascii_case(&'n')) =>
        {
            Some(Message::PresentationNext)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && matches!(key, keyboard::Key::Named(keyboard::key::Named::ArrowLeft)) =>
        {
            Some(Message::PresentationPrevious)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && key
                .to_latin(physical_key)
                .is_some_and(|character| character.eq_ignore_ascii_case(&'b')) =>
        {
            Some(Message::PresentationPrevious)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && matches!(key, keyboard::Key::Named(keyboard::key::Named::Home)) =>
        {
            Some(Message::PresentationFirst)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && matches!(key, keyboard::Key::Named(keyboard::key::Named::End)) =>
        {
            Some(Message::PresentationLast)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) if modifiers.is_empty()
            && matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
        {
            Some(Message::PresentationExit)
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

fn watch_remote_image(url: &RemoteImageUrl) -> BoxStream<'static, Message> {
    let url = url.0.clone();

    iced::stream::channel(1, async move |mut output| {
        let result = fetch_remote_image(&url).await;
        let _ = output
            .send(Message::RemoteImageLoaded { url, result })
            .await;
    })
    .boxed()
}

async fn fetch_remote_image(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|error| format!("request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("server returned {}", response.status()));
    }

    if let Some(length) = response.content_length()
        && length > MAX_REMOTE_IMAGE_BYTES as u64
    {
        return Err(format!(
            "image is larger than {} MB",
            MAX_REMOTE_IMAGE_BYTES / 1024 / 1024
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("read failed: {error}"))?;

    if bytes.len() > MAX_REMOTE_IMAGE_BYTES {
        return Err(format!(
            "image is larger than {} MB",
            MAX_REMOTE_IMAGE_BYTES / 1024 / 1024
        ));
    }

    Ok(bytes.to_vec())
}

fn load_history(store: &HistoryStore) -> History {
    let mut history = store.load().unwrap_or_else(|error| {
        eprintln!("{error}");
        History::new()
    });
    if history.prune_missing() > 0 {
        save_history(store, &history);
    }
    history
}

fn load_config(store: &ConfigStore) -> Config {
    store.load().unwrap_or_else(|error| {
        eprintln!("{error}");
        Config::default()
    })
}

#[cfg(test)]
fn test_config_store(history_store: &HistoryStore) -> ConfigStore {
    ConfigStore::new(history_store.path().with_extension("config.toml"))
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
        Some(document) if state.edit_session.is_some() => {
            let preview = state.edit_preview.as_ref().unwrap_or(document);
            row![
                left_sidebar(state),
                editing_view(state, preview),
                navigation::view(
                    preview.parsed(),
                    state.active_toc_block_index,
                    Message::TocSelected
                )
            ]
            .into()
        }
        Some(_) if state.presentation_deck.is_some() => {
            let document = state
                .presentation_document
                .as_ref()
                .expect("presentation document");
            reader::view_with_search_and_remote_images(
                document,
                None::<fn(f32) -> Message>,
                Message::OpenLink,
                None,
                None,
                None,
                &state.remote_images,
            )
        }
        Some(document) if state.zen_mode.is_enabled() => {
            reader::view_with_search_and_remote_images(
                document,
                Some(Message::ReaderScrolled),
                Message::OpenLink,
                state.active_search_query(),
                state.active_search_line(),
                Some(Message::ToggleTask),
                &state.remote_images,
            )
        }
        Some(document) => {
            let reader = if let Some(secondary) = state.split_document() {
                split_reader(state, document, secondary)
            } else {
                reader::view_with_search_and_remote_images(
                    document,
                    Some(Message::ReaderScrolled),
                    Message::OpenLink,
                    state.active_search_query(),
                    state.active_search_line(),
                    Some(Message::ToggleTask),
                    &state.remote_images,
                )
            };

            row![
                left_sidebar(state),
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
    let layout = if state.zen_mode.is_enabled() || state.presentation_deck.is_some() {
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

fn left_sidebar(state: &PaperView) -> Element<'_, Message> {
    container(scrollable(
        column![history_section(state), workspace_search_section(state)].spacing(24),
    ))
    .width(300)
    .height(Fill)
    .padding([22, 18])
    .style(|_| theme::history_container())
    .into()
}

fn history_section(state: &PaperView) -> Element<'_, Message> {
    let mut content = column![text("History").size(14).color(theme::SHELL_TEXT)].spacing(12);

    if state.history.is_empty() {
        content = content.push(
            text("No recent files")
                .size(12)
                .color(theme::SHELL_TEXT_MUTED),
        );
    } else {
        for entry in state.history.entries() {
            content = content.push(
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
                .on_press(Message::OpenHistory(entry.path().to_path_buf())),
            );
        }
    }

    content.into()
}

fn workspace_search_section(state: &PaperView) -> Element<'_, Message> {
    let can_submit =
        !state.workspace_search_query.trim().is_empty() && !state.is_workspace_searching;
    let mut input = text_input("Workspace search", &state.workspace_search_query)
        .padding([7, 10])
        .size(13)
        .width(Fill)
        .style(theme::search_input)
        .on_input(Message::WorkspaceSearchQueryChanged);

    if can_submit {
        input = input.on_submit(Message::SubmitWorkspaceSearch);
    }

    let search_button = button(
        text(if state.is_workspace_searching {
            "Searching"
        } else {
            "Find"
        })
        .size(13),
    )
    .padding([7, 12])
    .style(move |_, status| theme::header_action_button(state.is_workspace_searching, status));
    let search_button = if can_submit {
        search_button.on_press(Message::SubmitWorkspaceSearch)
    } else {
        search_button
    };

    let summary = workspace_search_summary(state);
    let mut content = column![
        text("Workspace").size(14).color(theme::SHELL_TEXT),
        row![input, search_button].spacing(6),
        text(summary).size(12).color(theme::SHELL_TEXT_MUTED)
    ]
    .spacing(10);

    if let Some(error) = &state.workspace_search_error {
        content = content.push(text(error).size(12).color(theme::SHELL_TEXT_MUTED));
    }

    for (index, search_match) in state.workspace_search_matches.iter().take(20).enumerate() {
        content = content.push(workspace_search_result(index, search_match));
    }

    content.into()
}

fn workspace_search_result(
    index: usize,
    search_match: &WorkspaceSearchMatch,
) -> Element<'_, Message> {
    let location = format!(
        "{}:{}:{}",
        search_match.path.display(),
        search_match.line_number,
        search_match.column
    );

    button(
        column![
            text(location).size(12).color(theme::SHELL_TEXT),
            text(search_match.line.trim().to_owned())
                .size(11)
                .color(theme::SHELL_TEXT_MUTED)
        ]
        .spacing(3),
    )
    .padding([8, 10])
    .width(Fill)
    .style(|_, status| theme::history_item_button(status))
    .on_press(Message::OpenWorkspaceSearchResult(index))
    .into()
}

fn workspace_search_summary(state: &PaperView) -> String {
    if state.is_workspace_searching {
        return "Searching current workspace".to_owned();
    }

    if state.workspace_search_query.trim().is_empty() {
        return "Search files with ripgrep".to_owned();
    }

    match state.workspace_search_matches.len() {
        0 if state.workspace_search_error.is_none() => "No matches".to_owned(),
        1 => "1 match".to_owned(),
        len => format!("{len} matches"),
    }
}

fn editing_view<'a>(state: &'a PaperView, preview: &'a Document) -> Element<'a, Message> {
    let dirty_label = state.edit_session.as_ref().map_or("Clean", |session| {
        if session.is_dirty() {
            "Unsaved"
        } else {
            "Saved"
        }
    });

    let editor = column![
        row![
            text("Editor").size(14).color(theme::SHELL_TEXT),
            text(dirty_label).size(12).color(theme::SHELL_TEXT_MUTED)
        ]
        .spacing(10),
        text_editor(&state.edit_content)
            .placeholder("Edit Markdown")
            .on_action(Message::EditSource)
            .highlight_with::<MarkdownHighlighter>((), markdown_highlight_format)
            .padding(14)
            .height(Fill)
    ]
    .spacing(10)
    .padding(16)
    .height(Fill);

    row![
        container(editor)
            .width(Length::FillPortion(1))
            .height(Fill)
            .style(|_| theme::editor_container()),
        container(reader::view_with_search_and_remote_images(
            preview,
            Some(Message::ReaderScrolled),
            Message::OpenLink,
            state.active_search_query(),
            state.active_search_line(),
            Some(Message::ToggleTask),
            &state.remote_images
        ))
        .width(Length::FillPortion(1))
        .height(Fill)
    ]
    .spacing(0)
    .into()
}

fn split_reader<'a>(
    state: &'a PaperView,
    document: &'a Document,
    secondary: &'a Document,
) -> Element<'a, Message> {
    responsive(move |size| {
        let (primary_width, secondary_width) = state.split_widths();
        let is_divider_active = state.is_split_dragging || state.is_split_divider_hovered();

        let panes = row![
            container(reader::view_with_search_and_remote_images(
                document,
                Some(Message::ReaderScrolled),
                Message::OpenLink,
                state.active_search_query(),
                state.active_search_line(),
                Some(Message::ToggleTask),
                &state.remote_images
            ))
            .width(Length::FillPortion(primary_width)),
            split_divider(is_divider_active),
            container(reader::split_view_with_remote_images(
                secondary,
                Message::OpenLink,
                &state.remote_images
            ))
            .width(Length::FillPortion(secondary_width))
        ]
        .spacing(0);

        mouse_area(panes)
            .on_move(move |position| Message::SplitDragMoved {
                x: position.x,
                width: size.width,
            })
            .into()
    })
    .into()
}

fn split_divider(is_active: bool) -> Element<'static, Message> {
    mouse_area(
        container(text(""))
            .width(8)
            .height(Fill)
            .style(move |_| theme::split_divider(is_active)),
    )
    .on_press(Message::SplitDragStarted)
    .on_release(Message::SplitDragEnded)
    .interaction(mouse::Interaction::ResizingHorizontally)
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
        || is_external_link_target(target)
        || PathBuf::from(target).is_absolute()
}

fn is_external_link_target(target: &str) -> bool {
    target.contains("://") || target.starts_with("mailto:") || target.starts_with("tel:")
}

fn resolve_local_document_link(
    target: &str,
    active_document: Option<&Document>,
) -> Option<PathBuf> {
    let trimmed = target.trim();
    if is_external_link_target(trimmed) || trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let path_part = trimmed.split_once('#').map_or(trimmed, |(path, _)| path);
    if path_part.is_empty() {
        return None;
    }

    let path = PathBuf::from(path_part);
    if !is_supported_document_path(&path) {
        return None;
    }

    if path.is_absolute() {
        return Some(path);
    }

    active_document
        .and_then(Document::path)
        .and_then(|active_path| active_path.parent())
        .map(|parent| parent.join(path))
}

fn is_supported_document_path(path: &PathBuf) -> bool {
    SupportedFileType::from_path(path).is_some()
}

fn is_tex_document_path(path: &Path) -> bool {
    SupportedFileType::from_path(path) == Some(SupportedFileType::Tex)
}

fn compile_tex_for_gui(
    path: &Path,
    config: &Config,
    open_after_compile: bool,
) -> Result<PathBuf, String> {
    let mut input = paperview_core::TexCompileInput::new(path);
    if let Some(compiler_path) = &config.tex_compiler_path {
        input = input.with_compiler_path(compiler_path);
    }

    let artifact = paperview_core::compile_tex(&input).map_err(|error| error.to_string())?;
    let output_path = artifact.output_path().to_path_buf();

    if open_after_compile {
        open_link_target(output_path.display().to_string())?;
    }

    Ok(output_path)
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
    let subtitle = if let Some(deck) = &state.presentation_deck {
        let title = deck
            .slides()
            .get(state.presentation_slide_index)
            .map_or("Untitled Slide", paperview_core::Slide::title);
        format!(
            "Slide {}/{} - {title}",
            state.presentation_slide_index + 1,
            deck.len()
        )
    } else {
        match &state.status {
            Status::Empty => format!(
                "No document open - {}",
                state.history_store.path().display()
            ),
            Status::Loaded(path) => path.display().to_string(),
            Status::CompilingTex(path) => format!("Compiling {}", path.display()),
            Status::CompiledTex { source, output } => {
                format!("Compiled {} -> {}", source.display(), output.display())
            }
            Status::Hovering(path) => format!("Drop to open {}", path.display()),
            Status::Error(error) => error.clone(),
        }
    };

    let is_split_enabled = state.split_view.secondary_index().is_some();
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
    let is_editing = state.edit_session.is_some();
    let edit_button = button(text(if is_editing { "View" } else { "Edit" }).size(13))
        .padding([7, 12])
        .style(move |_, status| theme::header_action_button(is_editing, status));
    let edit_button = if state.documents.active().is_some() {
        edit_button.on_press(Message::ToggleEdit)
    } else {
        edit_button
    };
    let can_save = state
        .edit_session
        .as_ref()
        .is_some_and(EditSession::is_dirty);
    let save_button = button(text("Save").size(13))
        .padding([7, 12])
        .style(move |_, status| theme::header_action_button(false, status));
    let save_button = if can_save {
        save_button.on_press(Message::SaveEdit)
    } else {
        save_button
    };
    let is_presenting = state.presentation_deck.is_some();
    let present_button = button(text(if is_presenting { "View" } else { "Present" }).size(13))
        .padding([7, 12])
        .style(move |_, status| theme::header_action_button(is_presenting, status));
    let present_button = if state.documents.active().is_some() {
        present_button.on_press(Message::TogglePresentation)
    } else {
        present_button
    };
    let previous_slide = button(text("<").size(13))
        .padding([7, 10])
        .style(move |_, status| theme::header_action_button(false, status));
    let previous_slide = if is_presenting && state.presentation_slide_index > 0 {
        previous_slide.on_press(Message::PresentationPrevious)
    } else {
        previous_slide
    };
    let next_slide = button(text(">").size(13))
        .padding([7, 10])
        .style(move |_, status| theme::header_action_button(false, status));
    let can_advance_presentation = state
        .presentation_deck
        .as_ref()
        .is_some_and(|deck| state.presentation_slide_index + 1 < deck.len());
    let next_slide = if can_advance_presentation {
        next_slide.on_press(Message::PresentationNext)
    } else {
        next_slide
    };
    let presentation_controls = if is_presenting {
        row![previous_slide, next_slide].spacing(6)
    } else {
        row![].spacing(0)
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
            presentation_controls,
            present_button,
            edit_button,
            save_button,
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
            let is_secondary = state.split_view.secondary_index() == Some(index);

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

            if state.split_view.secondary_index().is_some() && !is_active {
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
        Status::CompilingTex(_) => ("Compiling LaTeX", "Tectonic is generating a PDF."),
        Status::CompiledTex { .. } => ("LaTeX compiled", "Generated PDF opened externally."),
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
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::{
        ffi::OsString,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use iced::{
        Event,
        keyboard::{
            Key, Location, Modifiers,
            key::{Code, Named, Physical},
        },
        widget::text_editor,
    };
    use paperview_core::{
        Config, ConfigStore, Document, History, HistoryStore, ThemePreference,
        WorkspaceSearchMatch,
        parser::{Block, parse_markdown},
    };

    use super::{
        Message, PaperView, SplitResize, compile_tex_for_gui, is_split_divider_hit,
        normalized_scroll_progress, open_link_target, reader, remote_image_placeholders,
        resolve_link_target, resolve_local_document_link, runtime_event, search_scroll_progress,
        split_primary_width_from_cursor, title, update,
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
    fn gui_prunes_missing_history_entries_on_load() {
        let existing = temp_doc("history-existing.md", "# Existing");
        let missing = existing.with_file_name("history-missing.md");
        let store = temp_store("history-prune.toml");
        let mut history = History::new();
        history.record(paperview_core::FileEntry::new(&missing, "Missing"));
        history.record(paperview_core::FileEntry::new(&existing, "Existing"));
        store.save(&history).expect("save history");

        let state = PaperView::from_args_with_store([], store.clone());

        assert_eq!(state.history.entries().len(), 1);
        assert_eq!(state.history.entries()[0].path(), existing.as_path());
        assert_eq!(
            store.load().expect("load pruned history").entries().len(),
            1
        );

        fs::remove_file(existing).expect("remove existing history file");
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
    fn file_changed_reloads_split_document_without_activating_it() {
        let first = temp_doc("gui-split-reload-first.md", "# First\n\nOne.");
        let second = temp_doc("gui-split-reload-second.md", "# Before\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-reload.toml"),
        );
        let _ = state.open_path(second.clone());
        state.select_tab(0);
        state.toggle_split();

        fs::write(&second, "# After\n\nUpdated.").expect("rewrite split document");
        apply(&mut state, Message::FileChanged(second.clone()));

        assert_eq!(state.documents.active().map(Document::title), Some("First"));
        assert_eq!(state.split_document().map(Document::title), Some("After"));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &second)
        );

        fs::remove_file(first).expect("remove first document");
        fs::remove_file(second).expect("remove second document");
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
    fn workspace_search_query_clears_results_when_empty() {
        let mut state = PaperView::from_args_with_store([], temp_store("workspace-empty.toml"));
        state.workspace_search_matches = vec![WorkspaceSearchMatch {
            path: std::path::PathBuf::from("README.md"),
            line_number: 1,
            column: 1,
            line: "PaperView".to_owned(),
        }];
        state.is_workspace_searching = true;
        state.workspace_search_error = Some("old error".to_owned());

        apply(
            &mut state,
            Message::WorkspaceSearchQueryChanged("   ".to_owned()),
        );

        assert!(state.workspace_search_matches.is_empty());
        assert!(!state.is_workspace_searching);
        assert!(state.workspace_search_error.is_none());
    }

    #[test]
    fn workspace_search_finished_records_matches_and_errors() {
        let mut state = PaperView::from_args_with_store([], temp_store("workspace-finished.toml"));
        let search_match = WorkspaceSearchMatch {
            path: std::path::PathBuf::from("README.md"),
            line_number: 2,
            column: 4,
            line: "PaperView".to_owned(),
        };

        apply(
            &mut state,
            Message::WorkspaceSearchFinished(Ok(vec![search_match.clone()])),
        );

        assert_eq!(state.workspace_search_matches, vec![search_match]);
        assert!(!state.is_workspace_searching);
        assert!(state.workspace_search_error.is_none());

        apply(
            &mut state,
            Message::WorkspaceSearchFinished(Err("rg exploded".to_owned())),
        );

        assert!(state.workspace_search_matches.is_empty());
        assert_eq!(state.workspace_search_error.as_deref(), Some("rg exploded"));
        assert!(matches!(state.status, super::Status::Error(ref error) if error == "rg exploded"));
    }

    #[test]
    fn workspace_search_result_opens_document_near_match() {
        let path = temp_doc(
            "workspace-result-open.md",
            "# First\n\nOne.\n\n# Second\n\nNeedle.\n\n# Third",
        );
        let mut state =
            PaperView::from_args_with_store([], temp_store("workspace-result-open.toml"));
        state.workspace_search_matches = vec![WorkspaceSearchMatch {
            path: path.clone(),
            line_number: 6,
            column: 1,
            line: "Needle.".to_owned(),
        }];

        let _task = update(&mut state, Message::OpenWorkspaceSearchResult(0));

        assert_eq!(state.documents.active().map(Document::title), Some("First"));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn split_scroll_progress_is_bounded() {
        assert_eq!(normalized_scroll_progress(-1.0), 0.0);
        assert_eq!(normalized_scroll_progress(f32::NAN), 0.0);
        assert_eq!(normalized_scroll_progress(0.5), 0.5);
        assert_eq!(normalized_scroll_progress(2.0), 1.0);
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

    #[cfg(unix)]
    #[test]
    fn gui_tex_compile_uses_configured_compiler() {
        let stem = unique_test_stem("gui-tex-compile");
        let tex_path = std::env::temp_dir().join(format!("{stem}.tex"));
        let compiler_path = fake_tectonic_compiler(&stem);
        fs::write(&tex_path, "\\documentclass{article}").expect("write tex fixture");

        let output = compile_tex_for_gui(
            &tex_path,
            &Config {
                tex_compiler_path: Some(compiler_path.clone()),
                ..Config::default()
            },
            false,
        )
        .expect("compile tex");

        assert_eq!(output, tex_path.with_extension("pdf"));
        assert!(output.exists());

        fs::remove_file(tex_path).expect("remove tex fixture");
        fs::remove_file(output).expect("remove pdf fixture");
        fs::remove_file(compiler_path).expect("remove fake compiler");
    }

    #[cfg(unix)]
    #[test]
    fn tex_open_starts_async_compile_and_finish_sets_status() {
        let stem = unique_test_stem("gui-drop-tex");
        let tex_path = std::env::temp_dir().join(format!("{stem}.tex"));
        let history_store = temp_store("gui-drop-tex-history.toml");
        let config_path = temp_doc("gui-drop-tex-config.toml", "");
        let config_store = ConfigStore::new(&config_path);
        fs::write(&tex_path, "\\documentclass{article}").expect("write tex fixture");
        let mut state = PaperView::from_args_with_stores([], history_store, config_store);

        let _ = state.open_tex_path(tex_path.clone(), false);

        assert_eq!(state.documents.len(), 0);
        assert!(
            matches!(state.status, super::Status::CompilingTex(ref source) if source == &tex_path)
        );

        let output = tex_path.with_extension("pdf");
        state.finish_tex_compile(tex_path.clone(), Ok(output.clone()));

        assert!(
            matches!(state.status, super::Status::CompiledTex { ref source, ref output }
                if source == &tex_path && output == &tex_path.with_extension("pdf"))
        );

        fs::remove_file(tex_path).expect("remove tex fixture");
        fs::remove_file(config_path).expect("remove config");
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
    fn edit_toggle_starts_session_for_active_document() {
        let path = temp_doc("edit-toggle.md", "# Draft\n\nBody");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("edit.toml"));

        apply(&mut state, Message::ToggleEdit);

        let session = state.edit_session.as_ref().expect("edit session");
        assert_eq!(session.buffer(), "# Draft\n\nBody");
        assert!(!session.is_dirty());
        assert_eq!(
            state.edit_preview.as_ref().map(Document::title),
            Some("Draft")
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn presentation_toggle_starts_deck_for_active_document() {
        let path = temp_doc(
            "gui-presentation-toggle.md",
            "# Intro\n\nOpening.\n\n---\n\n# Next\n\nSecond.",
        );
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("presentation-toggle.toml"),
        );

        apply(&mut state, Message::TogglePresentation);

        assert_eq!(
            state.presentation_deck.as_ref().map(|deck| deck.len()),
            Some(2)
        );
        assert_eq!(state.presentation_slide_index, 0);
        assert_eq!(
            state.presentation_document.as_ref().map(Document::title),
            Some("Intro")
        );
        assert_eq!(
            state
                .presentation_document
                .as_ref()
                .and_then(Document::path),
            Some(&path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn presentation_navigation_clamps_between_slides() {
        let path = temp_doc(
            "gui-presentation-navigation.md",
            "# One\n\n---\n\n# Two\n\n---\n\n# Three",
        );
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("presentation-navigation.toml"),
        );

        apply(&mut state, Message::TogglePresentation);
        apply(&mut state, Message::PresentationNext);
        apply(&mut state, Message::PresentationNext);
        apply(&mut state, Message::PresentationNext);

        assert_eq!(state.presentation_slide_index, 2);
        assert_eq!(
            state.presentation_document.as_ref().map(Document::title),
            Some("Three")
        );

        apply(&mut state, Message::PresentationPrevious);
        apply(&mut state, Message::PresentationPrevious);
        apply(&mut state, Message::PresentationPrevious);

        assert_eq!(state.presentation_slide_index, 0);
        assert_eq!(
            state.presentation_document.as_ref().map(Document::title),
            Some("One")
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn presentation_first_last_navigation_jumps_between_bounds() {
        let path = temp_doc(
            "gui-presentation-first-last.md",
            "# One\n\n---\n\n# Two\n\n---\n\n# Three",
        );
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("presentation-first-last.toml"),
        );

        apply(&mut state, Message::TogglePresentation);
        apply(&mut state, Message::PresentationLast);

        assert_eq!(state.presentation_slide_index, 2);
        assert_eq!(
            state.presentation_document.as_ref().map(Document::title),
            Some("Three")
        );

        apply(&mut state, Message::PresentationFirst);

        assert_eq!(state.presentation_slide_index, 0);
        assert_eq!(
            state.presentation_document.as_ref().map(Document::title),
            Some("One")
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn presentation_toggle_exits_to_reader() {
        let path = temp_doc("gui-presentation-exit.md", "# Intro\n\n---\n\n# Next");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("presentation-exit.toml"),
        );

        apply(&mut state, Message::TogglePresentation);
        apply(&mut state, Message::TogglePresentation);

        assert!(state.presentation_deck.is_none());
        assert!(state.presentation_document.is_none());
        assert_eq!(state.presentation_slide_index, 0);
        assert_eq!(state.active_toc_block_index, Some(0));
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn presentation_exit_message_only_exits_when_presenting() {
        let path = temp_doc(
            "gui-presentation-exit-message.md",
            "# Intro\n\n---\n\n# Next",
        );
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("presentation-exit-message.toml"),
        );

        apply(&mut state, Message::PresentationExit);

        assert!(state.presentation_deck.is_none());

        apply(&mut state, Message::TogglePresentation);
        apply(&mut state, Message::PresentationExit);

        assert!(state.presentation_deck.is_none());
        assert!(state.presentation_document.is_none());
        assert!(
            matches!(state.status, super::Status::Loaded(ref loaded_path) if loaded_path == &path)
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn saving_edit_session_updates_active_document() {
        let path = temp_doc("edit-save.md", "# Draft\n\nBody");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("edit-save.toml"));

        apply(&mut state, Message::ToggleEdit);
        state
            .edit_session
            .as_mut()
            .expect("edit session")
            .replace_buffer("# Saved\n\nUpdated");
        state.edit_content = text_editor::Content::with_text("# Saved\n\nUpdated");
        apply(&mut state, Message::SaveEdit);

        assert_eq!(state.documents.active().map(Document::title), Some("Saved"));
        assert_eq!(
            fs::read_to_string(&path).expect("read saved source"),
            "# Saved\n\nUpdated"
        );
        assert!(
            state
                .edit_session
                .as_ref()
                .is_some_and(|session| !session.is_dirty())
        );

        fs::remove_file(path).expect("remove test document");
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
    fn opening_document_tracks_remote_image_placeholders() {
        let path = temp_doc(
            "remote-image.md",
            "# Remote\n\n![Remote](https://example.com/image.png)\n\n![Local](assets/local.png)",
        );
        let state =
            PaperView::from_args_with_store([OsString::from(&path)], temp_store("remote.toml"));

        assert!(matches!(
            state.remote_images.get("https://example.com/image.png"),
            Some(reader::RemoteImage::Loading)
        ));
        assert!(!state.remote_images.contains_key("assets/local.png"));

        fs::remove_file(path).expect("remove remote image document");
    }

    #[test]
    fn toggling_task_updates_file_and_reloads_document() {
        let path = temp_doc("task-toggle.md", "# Tasks\n\n- [ ] Todo\n- [x] Done");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&path)],
            temp_store("task-toggle.toml"),
        );

        apply(&mut state, Message::ToggleTask(2));

        let updated = fs::read_to_string(&path).expect("read updated document");
        assert_eq!(updated, "# Tasks\n\n- [x] Todo\n- [x] Done");
        assert!(matches!(
            state.documents.active().and_then(|document| {
                document
                    .parsed()
                    .blocks
                    .iter()
                    .find_map(|block| match block {
                        Block::List { items, .. } => items.first().and_then(|item| item.checked),
                        _ => None,
                    })
            }),
            Some(true)
        ));

        fs::remove_file(path).expect("remove task toggle document");
    }

    #[test]
    fn remote_image_placeholders_deduplicate_urls() {
        let document = Document::from_source(
            "# Remote\n\n![One](https://example.com/image.png)\n\n![Two](https://example.com/image.png)",
        );
        let images = remote_image_placeholders(Some(&document));

        assert_eq!(images.len(), 1);
        assert!(matches!(
            images.get("https://example.com/image.png"),
            Some(reader::RemoteImage::Loading)
        ));
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

        assert_eq!(state.split_view.secondary_index(), None);
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
        assert_eq!(state.split_view.secondary_index(), Some(0));
        assert_eq!(state.split_document().map(Document::title), Some("First"));

        apply(&mut state, Message::ToggleSplit);

        assert_eq!(state.split_view.secondary_index(), None);

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
    fn gui_loads_zen_and_split_width_from_config() {
        let history_store = temp_store("gui-config-load-history.toml");
        let config_path = temp_doc("gui-config-load.toml", "");
        let config_store = ConfigStore::new(&config_path);
        config_store
            .save(&Config {
                schema_version: 1,
                theme: ThemePreference::Hybrid,
                zen_mode: true,
                split_primary_width: 65,
                tex_compiler_path: None,
            })
            .expect("save config");

        let state = PaperView::from_args_with_stores([], history_store, config_store);

        assert!(state.zen_mode.is_enabled());
        assert_eq!(state.split_widths(), (65, 35));
        assert_eq!(state.config.theme, ThemePreference::Hybrid);

        fs::remove_file(config_path).expect("remove config");
    }

    #[test]
    fn gui_persists_zen_and_keyboard_split_width() {
        let first = temp_doc("gui-config-first.md", "# First\n\nOne.");
        let second = temp_doc("gui-config-second.md", "# Second\n\nTwo.");
        let history_store = temp_store("gui-config-save-history.toml");
        let config_path = temp_doc("gui-config-save.toml", "");
        let config_store = ConfigStore::new(&config_path);
        config_store.ensure_exists().expect("ensure config");
        let mut state = PaperView::from_args_with_stores(
            [OsString::from(&first)],
            history_store,
            config_store.clone(),
        );
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::ResizeSplit(SplitResize::GrowPrimary));
        apply(&mut state, Message::ToggleZen);

        let config = config_store.load().expect("load saved config");
        assert!(config.zen_mode);
        assert_eq!(config.split_primary_width, 60);

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
        fs::remove_file(config_path).expect("remove config");
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
    fn split_drag_math_is_bounded() {
        assert_eq!(split_primary_width_from_cursor(0.0, 1000.0, 50), 30);
        assert_eq!(split_primary_width_from_cursor(500.0, 1000.0, 50), 50);
        assert_eq!(split_primary_width_from_cursor(650.0, 1000.0, 50), 65);
        assert_eq!(split_primary_width_from_cursor(1000.0, 1000.0, 50), 70);
        assert_eq!(split_primary_width_from_cursor(500.0, 0.0, 90), 70);
    }

    #[test]
    fn split_divider_hit_testing_uses_current_ratio() {
        assert!(is_split_divider_hit(500.0, 1000.0, 50));
        assert!(is_split_divider_hit(516.0, 1000.0, 50));
        assert!(!is_split_divider_hit(520.0, 1000.0, 50));
        assert!(is_split_divider_hit(650.0, 1000.0, 65));
    }

    #[test]
    fn split_drag_resizes_primary_width_when_started_on_divider() {
        let first = temp_doc("split-drag-first.md", "# First\n\nOne.");
        let second = temp_doc("split-drag-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-drag.toml"),
        );
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::ToggleSplit);

        apply(
            &mut state,
            Message::SplitDragMoved {
                x: 500.0,
                width: 1000.0,
            },
        );
        apply(&mut state, Message::SplitDragStarted);
        apply(
            &mut state,
            Message::SplitDragMoved {
                x: 650.0,
                width: 1000.0,
            },
        );
        apply(&mut state, Message::SplitDragEnded);

        assert_eq!(state.split_widths(), (65, 35));
        assert!(!state.is_split_dragging);

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
    }

    #[test]
    fn gui_persists_dragged_split_width() {
        let first = temp_doc("gui-config-drag-first.md", "# First\n\nOne.");
        let second = temp_doc("gui-config-drag-second.md", "# Second\n\nTwo.");
        let history_store = temp_store("gui-config-drag-history.toml");
        let config_path = temp_doc("gui-config-drag.toml", "");
        let config_store = ConfigStore::new(&config_path);
        config_store.ensure_exists().expect("ensure config");
        let mut state = PaperView::from_args_with_stores(
            [OsString::from(&first)],
            history_store,
            config_store.clone(),
        );
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));
        apply(&mut state, Message::ToggleSplit);

        apply(&mut state, Message::SplitDragStarted);
        apply(
            &mut state,
            Message::SplitDragMoved {
                x: 650.0,
                width: 1000.0,
            },
        );
        apply(&mut state, Message::SplitDragEnded);

        assert_eq!(
            config_store
                .load()
                .expect("load saved config")
                .split_primary_width,
            65
        );

        fs::remove_file(first).expect("remove first test document");
        fs::remove_file(second).expect("remove second test document");
        fs::remove_file(config_path).expect("remove config");
    }

    #[test]
    fn split_drag_requires_enabled_split() {
        let first = temp_doc("split-drag-disabled-first.md", "# First\n\nOne.");
        let second = temp_doc("split-drag-disabled-second.md", "# Second\n\nTwo.");
        let mut state = PaperView::from_args_with_store(
            [OsString::from(&first)],
            temp_store("split-drag-disabled.toml"),
        );
        apply(&mut state, Message::OpenDroppedFiles(vec![second.clone()]));

        apply(&mut state, Message::SplitDragStarted);
        apply(
            &mut state,
            Message::SplitDragMoved {
                x: 650.0,
                width: 1000.0,
            },
        );

        assert_eq!(state.split_widths(), (50, 50));
        assert!(!state.is_split_dragging);

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
        assert_eq!(state.split_view.secondary_index(), Some(1));
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
        assert_eq!(state.split_view.secondary_index(), Some(1));
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
        assert_eq!(state.split_view.secondary_index(), Some(0));

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
        assert_eq!(state.split_view.secondary_index(), None);

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
        assert!(state.zen_mode.is_enabled());

        apply(&mut state, Message::ToggleZen);
        assert!(!state.zen_mode.is_enabled());
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
    fn command_e_maps_to_edit_toggle() {
        let message = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("e".into()),
                modified_key: Key::Character("e".into()),
                physical_key: Physical::Code(Code::KeyE),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND,
                text: Some("e".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );

        assert!(matches!(message, Some(Message::ToggleEdit)));
    }

    #[test]
    fn command_p_maps_to_presentation_toggle() {
        let message = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("p".into()),
                modified_key: Key::Character("p".into()),
                physical_key: Physical::Code(Code::KeyP),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND,
                text: Some("p".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );

        assert!(matches!(message, Some(Message::TogglePresentation)));
    }

    #[test]
    fn presentation_keys_map_to_slide_navigation() {
        let key_message = |key, code| {
            runtime_event(
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modified_key: Key::Unidentified,
                    physical_key: Physical::Code(code),
                    location: Location::Standard,
                    modifiers: Modifiers::NONE,
                    text: None,
                    repeat: false,
                }),
                iced::event::Status::Ignored,
                iced::window::Id::unique(),
            )
        };

        assert!(matches!(
            key_message(Key::Named(Named::ArrowRight), Code::ArrowRight),
            Some(Message::PresentationNext)
        ));
        assert!(matches!(
            key_message(Key::Named(Named::Space), Code::Space),
            Some(Message::PresentationNext)
        ));
        assert!(matches!(
            key_message(Key::Named(Named::ArrowLeft), Code::ArrowLeft),
            Some(Message::PresentationPrevious)
        ));
        assert!(matches!(
            key_message(Key::Named(Named::Home), Code::Home),
            Some(Message::PresentationFirst)
        ));
        assert!(matches!(
            key_message(Key::Named(Named::End), Code::End),
            Some(Message::PresentationLast)
        ));
        assert!(matches!(
            key_message(Key::Named(Named::Escape), Code::Escape),
            Some(Message::PresentationExit)
        ));
    }

    #[test]
    fn presentation_letter_keys_map_to_slide_navigation() {
        let character_message = |character: &str, code| {
            runtime_event(
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: Key::Character(character.into()),
                    modified_key: Key::Character(character.into()),
                    physical_key: Physical::Code(code),
                    location: Location::Standard,
                    modifiers: Modifiers::NONE,
                    text: Some(character.into()),
                    repeat: false,
                }),
                iced::event::Status::Ignored,
                iced::window::Id::unique(),
            )
        };

        assert!(matches!(
            character_message("n", Code::KeyN),
            Some(Message::PresentationNext)
        ));
        assert!(matches!(
            character_message("b", Code::KeyB),
            Some(Message::PresentationPrevious)
        ));
    }

    #[test]
    fn command_s_maps_to_edit_save() {
        let message = runtime_event(
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Character("s".into()),
                modified_key: Key::Character("s".into()),
                physical_key: Physical::Code(Code::KeyS),
                location: Location::Standard,
                modifiers: Modifiers::COMMAND,
                text: Some("s".into()),
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        );

        assert!(matches!(message, Some(Message::SaveEdit)));
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
    fn local_document_links_open_as_gui_tabs() {
        let source = temp_doc(
            "local-link-source.md",
            "# Source\n\n[Target](local-link-target.md)",
        );
        let target = source
            .parent()
            .expect("source parent")
            .join("local-link-target.md");
        fs::write(&target, "# Target\n\nOpened in PaperView.").expect("write target document");
        let mut state =
            PaperView::from_args_with_store([OsString::from(&source)], temp_store("link.toml"));

        apply(
            &mut state,
            Message::OpenLink("local-link-target.md".to_owned()),
        );

        assert_eq!(state.documents.len(), 2);
        assert_eq!(
            state.documents.active().and_then(Document::path),
            Some(&target)
        );

        fs::remove_file(source).expect("remove source document");
        fs::remove_file(target).expect("remove target document");
    }

    #[test]
    fn external_links_do_not_resolve_as_local_documents() {
        let source = temp_doc("external-link-source.md", "# Source");
        let state =
            PaperView::from_args_with_store([OsString::from(&source)], temp_store("external.toml"));

        assert_eq!(
            resolve_local_document_link("https://example.com/guide.md", state.documents.active()),
            None
        );
        assert_eq!(
            resolve_local_document_link("mailto:team@example.com", state.documents.active()),
            None
        );

        fs::remove_file(source).expect("remove source document");
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

    fn unique_test_stem(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        format!("paperview-{prefix}-{nanos}")
    }

    #[cfg(unix)]
    fn fake_tectonic_compiler(stem: &str) -> PathBuf {
        let compiler_path = std::env::temp_dir().join(format!("{stem}-tectonic"));
        let script = r#"#!/bin/sh
set -eu
outdir=""
input=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --outdir)
      outdir="$2"
      shift 2
      ;;
    *)
      input="$1"
      shift
      ;;
  esac
done
stem=$(basename "$input" .tex)
printf 'fake pdf' > "$outdir/$stem.pdf"
printf 'compiled from fake gui tectonic'
"#;
        fs::write(&compiler_path, script).expect("write fake compiler");
        let mut permissions = fs::metadata(&compiler_path)
            .expect("read fake compiler metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&compiler_path, permissions).expect("set fake compiler executable");
        compiler_path
    }
}
