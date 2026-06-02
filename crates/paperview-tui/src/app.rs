use std::{
    fs, io,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use paperview_core::{
    Bookmark, BookmarkStore, Bookmarks, Config, ConfigStore, Document, EditSession, FileEntry,
    FileWatcher, History, HistoryStore, OpenDocuments, PresentationDeck, SearchMatch, SplitResize,
    SplitViewState, WatchEvent, WorkspaceSearchMatch, ZenModeState,
    parser::{Block as MarkdownBlock, TocItem},
    presentation_deck, toggle_task_line_source, watch_file,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{render, theme};

const EDIT_VIEWPORT_LINES: usize = 16;
const EDIT_PAGE_LINES: usize = 12;

pub fn run(document: Document) -> io::Result<()> {
    run_documents(vec![document])
}

pub fn run_documents(documents: Vec<Document>) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = ReaderApp::new_documents(documents).run(&mut terminal);
    ratatui::restore();
    result
}

pub fn run_dashboard() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = DashboardApp::new(HistoryStore::default()).run(&mut terminal);
    ratatui::restore();
    result
}

pub fn run_workspace_search(
    query: String,
    root: std::path::PathBuf,
    matches: Vec<WorkspaceSearchMatch>,
) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result =
        WorkspaceSearchApp::new(query, root, matches, HistoryStore::default()).run(&mut terminal);
    ratatui::restore();
    result
}

pub fn run_bookmarks() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result =
        BookmarkApp::new(BookmarkStore::default(), HistoryStore::default()).run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReaderStartupProbe {
    pub(crate) document_count: usize,
    pub(crate) rendered_lines: usize,
    pub(crate) toc_items: usize,
    pub(crate) watcher_enabled: bool,
}

pub(crate) fn probe_reader_startup(documents: Vec<Document>) -> ReaderStartupProbe {
    let app = ReaderApp::new_documents(documents);

    ReaderStartupProbe {
        document_count: app.documents.len(),
        rendered_lines: app.document_lines.len(),
        toc_items: app.toc.len(),
        watcher_enabled: app._watcher.is_some(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardStartupProbe {
    pub(crate) history_entries: usize,
    pub(crate) selected_entry: Option<usize>,
}

pub(crate) fn probe_dashboard_startup() -> DashboardStartupProbe {
    let app = DashboardApp::new(HistoryStore::default());

    DashboardStartupProbe {
        history_entries: app.history.entries().len(),
        selected_entry: app.list_state.selected(),
    }
}

struct ReaderApp {
    config: Config,
    config_store: ConfigStore,
    history_store: HistoryStore,
    documents: OpenDocuments,
    document_lines: Vec<String>,
    split_view: SplitViewState,
    split_document_lines: Vec<String>,
    split_scroll: u16,
    block_line_starts: Vec<render::BlockLineStart>,
    toc: Vec<TocItem>,
    toc_selected_index: Option<usize>,
    focus: ReaderFocus,
    zen_mode: ZenModeState,
    scroll: u16,
    status: Option<String>,
    search_mode: SearchMode,
    search_query: String,
    search_matches: Vec<SearchMatch>,
    search_selected_index: Option<usize>,
    open_path_mode: OpenPathMode,
    open_path_input: String,
    edit_mode: EditMode,
    edit_session: Option<EditSession>,
    edit_buffer: String,
    edit_cursor: usize,
    edit_scroll: u16,
    edit_preview_lines: Vec<String>,
    edit_preview_scroll: u16,
    edit_preview_visible: bool,
    edit_discard_pending: bool,
    presentation_mode: PresentationMode,
    presentation_deck: Option<PresentationDeck>,
    presentation_slide_index: usize,
    presentation_lines: Vec<String>,
    _watcher: Option<FileWatcher>,
    watch_receiver: Option<Receiver<WatchEvent>>,
    _split_watcher: Option<FileWatcher>,
    split_watch_receiver: Option<Receiver<WatchEvent>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReaderFocus {
    Reader,
    Toc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchMode {
    Inactive,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenPathMode {
    Inactive,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditMode {
    Inactive,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresentationMode {
    Inactive,
    Presenting,
}

impl ReaderApp {
    fn new(document: Document) -> Self {
        Self::new_documents(vec![document])
    }

    fn new_documents(documents: Vec<Document>) -> Self {
        Self::new_documents_with_config(documents, default_config_store())
    }

    fn new_documents_with_config(documents: Vec<Document>, config_store: ConfigStore) -> Self {
        let (config, config_status) = load_config(&config_store);
        let mut open_documents = OpenDocuments::new();
        for document in documents {
            open_documents.open_or_activate(document);
        }
        if open_documents.is_empty() {
            open_documents.open_or_activate(Document::from_source("No document loaded."));
        }
        open_documents.select(0);

        let active = open_documents.active().expect("active document");
        let rendered = render::render_document_with_anchors(active);
        let toc = active.parsed().toc();
        let (watcher, watch_receiver, watch_status) = watch_document(active);
        let status = config_status.or(watch_status);

        Self {
            split_view: SplitViewState::new(config.split_primary_width),
            zen_mode: ZenModeState::new(config.zen_mode),
            config,
            config_store,
            history_store: default_history_store(),
            documents: open_documents,
            document_lines: rendered.lines,
            split_document_lines: Vec::new(),
            split_scroll: 0,
            block_line_starts: rendered.block_line_starts,
            toc_selected_index: initial_toc_selection(&toc),
            toc,
            focus: ReaderFocus::Reader,
            scroll: 0,
            status,
            search_mode: SearchMode::Inactive,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_selected_index: None,
            open_path_mode: OpenPathMode::Inactive,
            open_path_input: String::new(),
            edit_mode: EditMode::Inactive,
            edit_session: None,
            edit_buffer: String::new(),
            edit_cursor: 0,
            edit_scroll: 0,
            edit_preview_lines: Vec::new(),
            edit_preview_scroll: 0,
            edit_preview_visible: true,
            edit_discard_pending: false,
            presentation_mode: PresentationMode::Inactive,
            presentation_deck: None,
            presentation_slide_index: 0,
            presentation_lines: Vec::new(),
            _watcher: watcher,
            watch_receiver,
            _split_watcher: None,
            split_watch_receiver: None,
        }
    }

    fn new_at_source_line(document: Document, line_number: usize) -> Self {
        Self::new_at_source_line_with_status(
            document,
            line_number,
            format!("Opened search result at line {line_number}"),
        )
    }

    fn new_at_source_line_with_status(
        document: Document,
        line_number: usize,
        status: String,
    ) -> Self {
        let mut app = Self::new(document);
        app.scroll = line_number.saturating_sub(1) as u16;
        app.scroll = app.scroll.min(app.max_scroll());
        app.status = Some(status);
        app
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_watch_events();

            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if self.search_mode == SearchMode::Editing {
                    self.handle_search_key(key.code);
                    continue;
                }
                if self.open_path_mode == OpenPathMode::Editing {
                    self.handle_open_path_key(key.code);
                    continue;
                }
                if self.edit_mode == EditMode::Editing {
                    self.handle_edit_key(key);
                    continue;
                }
                if self.presentation_mode == PresentationMode::Presenting {
                    self.handle_presentation_key(key.code);
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('/') => self.start_search(),
                    KeyCode::Char('o') => self.start_open_path(),
                    KeyCode::Char('e') => self.start_editing(),
                    KeyCode::Char('p') => self.start_presentation(),
                    KeyCode::Char('n') => self.select_next_search_match(),
                    KeyCode::Char('N') => self.select_previous_search_match(),
                    KeyCode::Char(']') => self.select_next_tab(),
                    KeyCode::Char('[') => self.select_previous_tab(),
                    KeyCode::Char('}') => self.select_next_split_tab(),
                    KeyCode::Char('{') => self.select_previous_split_tab(),
                    KeyCode::Char('>') => self.grow_split_primary(),
                    KeyCode::Char('<') => self.shrink_split_primary(),
                    KeyCode::Char('\\') => self.toggle_split(),
                    KeyCode::Char('z') => self.toggle_zen(),
                    KeyCode::Char(' ') => self.toggle_task_at_scroll(),
                    KeyCode::Char('x') if self.close_active_tab() => return Ok(()),
                    KeyCode::Char('x') => {}
                    KeyCode::Tab => self.toggle_focus(),
                    KeyCode::Char('j') | KeyCode::Down => self.move_down(),
                    KeyCode::Char('k') | KeyCode::Up => self.move_up(),
                    KeyCode::Enter => self.jump_to_selected_toc(),
                    KeyCode::Char('g') if self.focus == ReaderFocus::Reader => {
                        self.scroll = 0;
                        self.sync_split_scroll();
                    }
                    KeyCode::Char('G') if self.focus == ReaderFocus::Reader => {
                        self.scroll_to_bottom();
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let [header, body] =
            Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(frame.area());
        let (main, toc) = if self.zen_mode.is_enabled()
            || self.presentation_mode == PresentationMode::Presenting
        {
            (body, None)
        } else {
            let [main, toc] =
                Layout::horizontal([Constraint::Min(50), Constraint::Length(32)]).areas(body);
            (main, Some(toc))
        };
        let reader_areas = if self.edit_mode == EditMode::Editing && self.edit_preview_visible {
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas::<2>(main)
                .to_vec()
        } else if self.split_view.is_enabled()
            && !self.zen_mode.is_enabled()
            && self.edit_mode == EditMode::Inactive
            && self.presentation_mode == PresentationMode::Inactive
        {
            let (primary_width, secondary_width) = self.split_widths();
            Layout::horizontal([
                Constraint::Percentage(primary_width),
                Constraint::Percentage(secondary_width),
            ])
            .areas::<2>(main)
            .to_vec()
        } else {
            vec![main]
        };

        frame.render_widget(
            Paragraph::new(self.header_lines())
                .style(theme::shell())
                .block(Block::default().borders(Borders::BOTTOM)),
            header,
        );

        if self.edit_mode == EditMode::Editing {
            frame.render_widget(
                Paragraph::new(Text::from(edit_buffer_text(
                    &self.edit_buffer,
                    self.edit_cursor,
                )))
                .block(Block::default().title("Editor").borders(Borders::ALL))
                .style(theme::reader())
                .scroll((self.edit_scroll, 0))
                .wrap(Wrap { trim: false }),
                reader_areas[0],
            );
            if self.edit_preview_visible {
                frame.render_widget(
                    Paragraph::new(Text::from(document_text(
                        &self.edit_preview_lines,
                        SearchHighlights::default(),
                    )))
                    .block(Block::default().title("Preview").borders(Borders::ALL))
                    .style(theme::reader())
                    .scroll((self.edit_preview_scroll, 0))
                    .wrap(Wrap { trim: false }),
                    reader_areas[1],
                );
            }
        } else if self.presentation_mode == PresentationMode::Presenting {
            frame.render_widget(
                Paragraph::new(Text::from(document_text(
                    &self.presentation_lines,
                    SearchHighlights::default(),
                )))
                .block(
                    Block::default()
                        .title(self.presentation_pane_title())
                        .borders(Borders::ALL),
                )
                .style(theme::reader())
                .scroll((self.scroll, 0))
                .wrap(Wrap { trim: false }),
                reader_areas[0],
            );
        } else {
            frame.render_widget(
                Paragraph::new(Text::from(document_text(
                    &self.document_lines,
                    self.search_highlights(),
                )))
                .block(Block::default().title("Reader").borders(Borders::ALL))
                .style(theme::reader())
                .scroll((self.scroll, 0))
                .wrap(Wrap { trim: false }),
                reader_areas[0],
            );
        }

        if let Some(split_index) = self.split_view.secondary_index() {
            if self.zen_mode.is_enabled()
                || self.edit_mode == EditMode::Editing
                || self.presentation_mode == PresentationMode::Presenting
            {
                return;
            }
            let title = self
                .documents
                .iter()
                .find_map(|(index, document)| (index == split_index).then_some(document.title()))
                .unwrap_or("Split");
            frame.render_widget(
                Paragraph::new(Text::from(document_text(
                    &self.split_document_lines,
                    SearchHighlights::default(),
                )))
                .block(
                    Block::default()
                        .title(format!("Side: {title}"))
                        .borders(Borders::ALL),
                )
                .style(theme::reader())
                .scroll((self.split_scroll, 0))
                .wrap(Wrap { trim: false }),
                reader_areas[1],
            );
        }

        if let Some(toc) = toc {
            frame.render_widget(
                Paragraph::new(render::render_toc_text(
                    &self.toc,
                    self.active_toc_block_index(),
                    self.toc_selected_index,
                    self.focus == ReaderFocus::Toc,
                ))
                .block(
                    Block::default()
                        .title(if self.focus == ReaderFocus::Toc {
                            "On this page [active]"
                        } else {
                            "On this page"
                        })
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: true }),
                toc,
            );
        }
    }

    fn move_down(&mut self) {
        match self.focus {
            ReaderFocus::Reader => self.scroll_down(),
            ReaderFocus::Toc => self.select_next_toc(),
        }
    }

    fn move_up(&mut self) {
        match self.focus {
            ReaderFocus::Reader => self.scroll_up(),
            ReaderFocus::Toc => self.select_previous_toc(),
        }
    }

    fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1).min(self.max_scroll());
        self.sync_split_scroll();
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
        self.sync_split_scroll();
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll = self.max_scroll();
        self.sync_split_scroll();
    }

    fn start_search(&mut self) {
        self.search_mode = SearchMode::Editing;
        self.search_query.clear();
        self.status = Some("Search: type a query, Enter jumps, Esc cancels".to_owned());
    }

    fn handle_search_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.search_mode = SearchMode::Inactive;
                self.status = None;
            }
            KeyCode::Enter => self.submit_search(),
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(character) => {
                self.search_query.push(character);
            }
            _ => {}
        }
    }

    fn start_open_path(&mut self) {
        self.open_path_mode = OpenPathMode::Editing;
        self.open_path_input.clear();
        self.status = Some("Open: paste or type a file path, Enter opens, Esc cancels".to_owned());
    }

    fn handle_open_path_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.open_path_mode = OpenPathMode::Inactive;
                self.status = None;
            }
            KeyCode::Enter => self.submit_open_path(),
            KeyCode::Backspace => {
                self.open_path_input.pop();
            }
            KeyCode::Char(character) => {
                self.open_path_input.push(character);
            }
            _ => {}
        }
    }

    fn submit_open_path(&mut self) {
        self.open_path_mode = OpenPathMode::Inactive;
        let path = self.open_path_input.trim().to_owned();
        if path.is_empty() {
            self.status = Some("Open path is empty".to_owned());
            return;
        }

        self.open_path(PathBuf::from(path));
    }

    fn start_editing(&mut self) {
        let session = EditSession::from_document(self.active_document());
        self.edit_buffer = session.buffer().to_owned();
        self.edit_cursor = self.edit_buffer.len();
        self.edit_scroll = 0;
        self.edit_preview_scroll = 0;
        self.edit_preview_visible = true;
        self.edit_discard_pending = false;
        self.edit_session = Some(session);
        self.refresh_edit_preview();
        self.ensure_edit_cursor_visible();
        self.edit_mode = EditMode::Editing;
        self.focus = ReaderFocus::Reader;
        self.scroll = 0;
        self.status =
            Some("Editing: arrows/PageUp/PageDown move, Ctrl+S save, Esc view".to_owned());
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.save_edit();
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P'))
        {
            self.toggle_edit_preview();
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Up => self.scroll_edit_preview(-1),
                KeyCode::Down => self.scroll_edit_preview(1),
                KeyCode::PageUp => self.scroll_edit_preview_page(-1),
                KeyCode::PageDown => self.scroll_edit_preview_page(1),
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.request_stop_editing();
            }
            KeyCode::Enter => self.insert_edit_char('\n'),
            KeyCode::Backspace => self.backspace_edit_char(),
            KeyCode::Delete => self.delete_edit_char(),
            KeyCode::Left => self.move_edit_cursor_left(),
            KeyCode::Right => self.move_edit_cursor_right(),
            KeyCode::Up => self.move_edit_cursor_vertical(-1),
            KeyCode::Down => self.move_edit_cursor_vertical(1),
            KeyCode::PageUp => self.page_edit_cursor(-1),
            KeyCode::PageDown => self.page_edit_cursor(1),
            KeyCode::Home => self.move_edit_cursor_to_line_start(),
            KeyCode::End => self.move_edit_cursor_to_line_end(),
            KeyCode::Char(character)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.insert_edit_char(character);
            }
            KeyCode::Char(_) => {}
            _ => {}
        }
    }

    fn insert_edit_char(&mut self, character: char) {
        self.edit_discard_pending = false;
        self.edit_cursor = clamp_to_char_boundary(&self.edit_buffer, self.edit_cursor);
        self.edit_buffer.insert(self.edit_cursor, character);
        self.edit_cursor += character.len_utf8();
        self.refresh_edit_session();
        self.ensure_edit_cursor_visible();
    }

    fn backspace_edit_char(&mut self) {
        self.edit_discard_pending = false;
        self.edit_cursor = clamp_to_char_boundary(&self.edit_buffer, self.edit_cursor);
        let Some(previous) = previous_char_boundary(&self.edit_buffer, self.edit_cursor) else {
            return;
        };
        self.edit_buffer.drain(previous..self.edit_cursor);
        self.edit_cursor = previous;
        self.refresh_edit_session();
        self.ensure_edit_cursor_visible();
    }

    fn delete_edit_char(&mut self) {
        self.edit_discard_pending = false;
        self.edit_cursor = clamp_to_char_boundary(&self.edit_buffer, self.edit_cursor);
        let Some(next) = next_char_boundary(&self.edit_buffer, self.edit_cursor) else {
            return;
        };
        self.edit_buffer.drain(self.edit_cursor..next);
        self.refresh_edit_session();
        self.ensure_edit_cursor_visible();
    }

    fn move_edit_cursor_left(&mut self) {
        if let Some(previous) = previous_char_boundary(&self.edit_buffer, self.edit_cursor) {
            self.edit_cursor = previous;
            self.ensure_edit_cursor_visible();
        }
    }

    fn move_edit_cursor_right(&mut self) {
        if let Some(next) = next_char_boundary(&self.edit_buffer, self.edit_cursor) {
            self.edit_cursor = next;
            self.ensure_edit_cursor_visible();
        }
    }

    fn move_edit_cursor_to_line_start(&mut self) {
        let (line, _) = cursor_line_column(&self.edit_buffer, self.edit_cursor);
        self.edit_cursor = cursor_offset_for_line_column(&self.edit_buffer, line, 0);
        self.ensure_edit_cursor_visible();
    }

    fn move_edit_cursor_to_line_end(&mut self) {
        let (line, _) = cursor_line_column(&self.edit_buffer, self.edit_cursor);
        self.edit_cursor = cursor_offset_for_line_end(&self.edit_buffer, line);
        self.ensure_edit_cursor_visible();
    }

    fn move_edit_cursor_vertical(&mut self, delta: isize) {
        let (line, column) = cursor_line_column(&self.edit_buffer, self.edit_cursor);
        let target_line = if delta.is_negative() {
            line.saturating_sub(delta.unsigned_abs())
        } else {
            line.saturating_add(delta as usize)
                .min(edit_line_count(&self.edit_buffer).saturating_sub(1))
        };
        self.edit_cursor = cursor_offset_for_line_column(&self.edit_buffer, target_line, column);
        self.ensure_edit_cursor_visible();
    }

    fn page_edit_cursor(&mut self, direction: isize) {
        let delta = direction.saturating_mul(EDIT_PAGE_LINES as isize);
        self.move_edit_cursor_vertical(delta);
    }

    fn toggle_edit_preview(&mut self) {
        self.edit_preview_visible = !self.edit_preview_visible;
        let state = if self.edit_preview_visible {
            "visible"
        } else {
            "hidden"
        };
        self.status = Some(format!("Editing preview {state}"));
    }

    fn scroll_edit_preview(&mut self, delta: isize) {
        self.edit_preview_scroll = offset_scroll(
            self.edit_preview_scroll,
            delta,
            edit_preview_max_scroll(&self.edit_preview_lines, EDIT_VIEWPORT_LINES),
        );
    }

    fn scroll_edit_preview_page(&mut self, direction: isize) {
        self.scroll_edit_preview(direction.saturating_mul(EDIT_PAGE_LINES as isize));
    }

    fn ensure_edit_cursor_visible(&mut self) {
        let (line, _) = cursor_line_column(&self.edit_buffer, self.edit_cursor);
        let current_scroll = usize::from(self.edit_scroll);
        if line < current_scroll {
            self.edit_scroll = line as u16;
        } else if line >= current_scroll + EDIT_VIEWPORT_LINES {
            self.edit_scroll = line.saturating_sub(EDIT_VIEWPORT_LINES - 1) as u16;
        }

        self.edit_scroll = self
            .edit_scroll
            .min(edit_max_scroll(&self.edit_buffer, EDIT_VIEWPORT_LINES));
    }

    fn refresh_edit_session(&mut self) {
        if let Some(session) = &mut self.edit_session {
            session.replace_buffer(self.edit_buffer.clone());
        }
        self.refresh_edit_preview();
    }

    fn refresh_edit_preview(&mut self) {
        self.edit_preview_lines = self.edit_session.as_ref().map_or_else(Vec::new, |session| {
            render::render_document_with_anchors(&session.preview_document()).lines
        });
        self.edit_preview_scroll = self.edit_preview_scroll.min(edit_preview_max_scroll(
            &self.edit_preview_lines,
            EDIT_VIEWPORT_LINES,
        ));
    }

    fn save_edit(&mut self) {
        let Some(session) = &mut self.edit_session else {
            self.status = Some("Enter Editing Mode before saving".to_owned());
            return;
        };

        session.replace_buffer(self.edit_buffer.clone());
        match session.save() {
            Ok(document) => {
                self.documents.replace_active(document);
                self.load_active_document(false);
                self.edit_session = self.documents.active().map(EditSession::from_document);
                self.refresh_edit_preview();
                self.edit_discard_pending = false;
                self.status = Some("Saved edits".to_owned());
            }
            Err(error) => {
                self.status = Some(error.to_string());
            }
        }
    }

    fn request_stop_editing(&mut self) -> bool {
        if !self.edit_is_dirty() {
            self.stop_editing();
            self.status = Some("Editing Mode closed".to_owned());
            return true;
        }

        if self.edit_discard_pending {
            self.stop_editing();
            self.status = Some("Discarded unsaved edits".to_owned());
            return true;
        }

        self.edit_discard_pending = true;
        self.status =
            Some("Unsaved edits. Press Esc again to discard or Ctrl+S to save.".to_owned());
        false
    }

    fn edit_is_dirty(&self) -> bool {
        self.edit_session
            .as_ref()
            .is_some_and(EditSession::is_dirty)
    }

    fn stop_editing(&mut self) {
        if self.edit_mode == EditMode::Inactive {
            return;
        }

        self.edit_mode = EditMode::Inactive;
        self.edit_session = None;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
        self.edit_scroll = 0;
        self.edit_preview_lines.clear();
        self.edit_preview_scroll = 0;
        self.edit_preview_visible = true;
        self.edit_discard_pending = false;
        self.load_active_document(false);
    }

    fn open_path(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(document) => {
                self.record_opened_document(&document);
                let index = self.documents.open_or_activate(document);
                self.ensure_split_target();
                self.load_active_document(true);
                self.status = Some(format!(
                    "Opened tab {}/{}: {}",
                    index + 1,
                    self.documents.len(),
                    self.active_document().title()
                ));
            }
            Err(error) => {
                self.status = Some(error.to_string());
            }
        }
    }

    fn record_opened_document(&mut self, document: &Document) {
        let mut history = self.history_store.load().unwrap_or_else(|error| {
            self.status = Some(error.to_string());
            History::new()
        });
        history.record_document(document);
        if let Err(error) = self.history_store.save(&history) {
            self.status = Some(error.to_string());
        }
    }

    fn submit_search(&mut self) {
        self.search_mode = SearchMode::Inactive;
        self.refresh_search_matches();

        if self.search_matches.is_empty() {
            self.search_selected_index = None;
            self.status = Some(format!("No matches for '{}'", self.search_query));
            return;
        }

        self.search_selected_index = Some(0);
        self.scroll_to_search_match(0);
        self.status = Some(self.search_status());
    }

    fn refresh_search_matches(&mut self) {
        self.search_matches = self.active_document().search(&self.search_query);
        self.search_selected_index =
            clamp_search_selection(self.search_selected_index, self.search_matches.len());
    }

    fn start_presentation(&mut self) {
        let deck = presentation_deck(self.active_document().source());
        if deck.is_empty() {
            self.status = Some("Presentation has no slides".to_owned());
            return;
        }

        self.presentation_deck = Some(deck);
        self.presentation_slide_index = 0;
        self.presentation_mode = PresentationMode::Presenting;
        self.focus = ReaderFocus::Reader;
        self.scroll = 0;
        self.refresh_presentation_slide();
        self.status =
            Some("Presentation Mode: Space/Right/n next, Left/b previous, Esc exits".to_owned());
    }

    fn handle_presentation_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => self.stop_presentation(),
            KeyCode::Right | KeyCode::Char(' ') | KeyCode::Char('n') => {
                self.select_next_presentation_slide();
            }
            KeyCode::Left | KeyCode::Char('b') => self.select_previous_presentation_slide(),
            KeyCode::Home => self.select_first_presentation_slide(),
            KeyCode::End => self.select_last_presentation_slide(),
            KeyCode::Char('g') => self.scroll = 0,
            KeyCode::Char('G') => self.scroll = self.max_presentation_scroll(),
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll = self
                    .scroll
                    .saturating_add(1)
                    .min(self.max_presentation_scroll());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn presentation_pane_title(&self) -> String {
        self.presentation_deck.as_ref().map_or_else(
            || "Presentation".to_owned(),
            |deck| {
                format!(
                    "Presentation {} / {}",
                    self.presentation_slide_index + 1,
                    deck.len()
                )
            },
        )
    }

    fn select_next_presentation_slide(&mut self) {
        let Some(deck) = &self.presentation_deck else {
            return;
        };
        if self.presentation_slide_index + 1 < deck.len() {
            self.presentation_slide_index += 1;
            self.refresh_presentation_slide();
        }
    }

    fn select_previous_presentation_slide(&mut self) {
        if self.presentation_slide_index > 0 {
            self.presentation_slide_index -= 1;
            self.refresh_presentation_slide();
        }
    }

    fn select_first_presentation_slide(&mut self) {
        if self.presentation_slide_index != 0 {
            self.presentation_slide_index = 0;
            self.refresh_presentation_slide();
        }
    }

    fn select_last_presentation_slide(&mut self) {
        let Some(deck) = &self.presentation_deck else {
            return;
        };
        let last_index = deck.len().saturating_sub(1);
        if self.presentation_slide_index != last_index {
            self.presentation_slide_index = last_index;
            self.refresh_presentation_slide();
        }
    }

    fn refresh_presentation_slide(&mut self) {
        self.presentation_lines = self
            .presentation_deck
            .as_ref()
            .and_then(|deck| deck.slides().get(self.presentation_slide_index))
            .map_or_else(Vec::new, |slide| {
                render::render_document_with_anchors(&Document::from_source(slide.source())).lines
            });
        self.scroll = 0;
    }

    fn stop_presentation(&mut self) {
        self.presentation_mode = PresentationMode::Inactive;
        self.presentation_deck = None;
        self.presentation_slide_index = 0;
        self.presentation_lines.clear();
        self.scroll = 0;
        self.status = Some("Presentation Mode closed".to_owned());
        self.sync_split_scroll();
    }

    fn max_presentation_scroll(&self) -> u16 {
        self.presentation_lines.len().saturating_sub(1) as u16
    }

    fn select_next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let next = self
            .search_selected_index
            .map_or(0, |index| (index + 1) % self.search_matches.len());
        self.search_selected_index = Some(next);
        self.scroll_to_search_match(next);
        self.status = Some(self.search_status());
    }

    fn select_previous_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let previous = self.search_selected_index.map_or(0, |index| {
            if index == 0 {
                self.search_matches.len() - 1
            } else {
                index - 1
            }
        });
        self.search_selected_index = Some(previous);
        self.scroll_to_search_match(previous);
        self.status = Some(self.search_status());
    }

    fn scroll_to_search_match(&mut self, index: usize) {
        if let Some(search_match) = self.search_matches.get(index) {
            self.scroll = (search_match.line_index as u16).min(self.max_scroll());
            self.focus = ReaderFocus::Reader;
            self.sync_split_scroll();
        }
    }

    fn search_status(&self) -> String {
        let Some(index) = self.search_selected_index else {
            return format!("No matches for '{}'", self.search_query);
        };

        format!(
            "Match {}/{} for '{}' - n/N moves",
            index + 1,
            self.search_matches.len(),
            self.search_query
        )
    }

    fn header_status(&self) -> Option<String> {
        if self.search_mode == SearchMode::Editing {
            return Some(format!("/{}", self.search_query));
        }
        if self.open_path_mode == OpenPathMode::Editing {
            return Some(format!("Open: {}", self.open_path_input));
        }
        if self.edit_mode == EditMode::Editing {
            let state = self.edit_session.as_ref().map_or("clean", |session| {
                if session.is_dirty() { "dirty" } else { "clean" }
            });
            let preview = if self.edit_preview_visible {
                "preview on"
            } else {
                "preview off"
            };
            return Some(format!(
                "Editing ({state}, {preview}) - Ctrl+S save, Ctrl+P preview, Esc view"
            ));
        }
        if self.presentation_mode == PresentationMode::Presenting {
            let Some(deck) = &self.presentation_deck else {
                return Some("Presentation Mode".to_owned());
            };
            let title = deck
                .slides()
                .get(self.presentation_slide_index)
                .map_or("Untitled Slide", paperview_core::Slide::title);
            return Some(format!(
                "Slide {}/{} - {title} - Space/Right next, Left previous, Esc view",
                self.presentation_slide_index + 1,
                deck.len()
            ));
        }

        self.status.clone()
    }

    fn search_highlights(&self) -> SearchHighlights<'_> {
        SearchHighlights {
            query: &self.search_query,
            matches: &self.search_matches,
            selected_index: self.search_selected_index,
        }
    }

    fn active_document(&self) -> &Document {
        self.documents.active().expect("active document")
    }

    fn header_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                format!(" PaperView - {} ", self.active_document().title()),
                theme::shell(),
            )),
            if self.zen_mode.is_enabled() {
                Line::from(Span::styled(" Zen Mode ".to_owned(), theme::zen_badge()))
            } else {
                tab_line(&self.documents)
            },
            Line::from(Span::styled(
                self.header_status().unwrap_or_else(|| {
                    "[e] edit  [o] open  [/] search  [Space] task  [z] zen  [\\] split  [</>] resize  [{/}] side  [[/]] tabs  [x] close  [Tab] toc  [q] quit"
                        .to_owned()
                }),
                theme::shell_muted(),
            )),
        ]
    }

    fn select_next_tab(&mut self) {
        let Some(active) = self.documents.active_index() else {
            return;
        };

        self.select_tab((active + 1) % self.documents.len());
    }

    fn select_previous_tab(&mut self) {
        let Some(active) = self.documents.active_index() else {
            return;
        };
        let previous = if active == 0 {
            self.documents.len() - 1
        } else {
            active - 1
        };

        self.select_tab(previous);
    }

    fn select_tab(&mut self, index: usize) {
        if self.documents.active_index() == Some(index) {
            return;
        }

        if self.edit_mode == EditMode::Editing && !self.request_stop_editing() {
            return;
        }

        self.stop_editing();
        self.documents.select(index);
        self.ensure_split_target();
        self.load_active_document(true);
        self.status = Some(format!(
            "Tab {}/{}: {}",
            index + 1,
            self.documents.len(),
            self.active_document().title()
        ));
    }

    fn close_active_tab(&mut self) -> bool {
        if self.edit_mode == EditMode::Editing && !self.request_stop_editing() {
            return false;
        }

        self.stop_editing();
        let Some(index) = self.documents.active_index() else {
            return true;
        };
        let title = self.active_document().title().to_owned();
        self.documents.close(index);

        if self.documents.is_empty() {
            return true;
        }

        self.ensure_split_target();
        self.load_active_document(true);
        self.status = Some(format!("Closed {title}"));
        false
    }

    fn toggle_split(&mut self) {
        self.split_view
            .toggle(self.documents.active_index(), self.documents.len());
        self.refresh_split_document();
        self.status = if self.split_view.is_enabled() {
            Some("Split View enabled".to_owned())
        } else {
            self.split_scroll = 0;
            self._split_watcher = None;
            self.split_watch_receiver = None;
            Some("Split View disabled".to_owned())
        };
    }

    fn grow_split_primary(&mut self) {
        self.resize_split(SplitResize::GrowPrimary);
    }

    fn shrink_split_primary(&mut self) {
        self.resize_split(SplitResize::ShrinkPrimary);
    }

    fn resize_split(&mut self, direction: SplitResize) {
        if self.split_view.resize(direction) {
            let (primary, secondary) = self.split_widths();
            self.status = Some(format!("Split View {primary}/{secondary}"));
            self.save_config();
        }
    }

    fn split_widths(&self) -> (u16, u16) {
        self.split_view.widths()
    }

    fn toggle_zen(&mut self) {
        self.zen_mode.toggle();
        if self.zen_mode.is_enabled() {
            self.focus = ReaderFocus::Reader;
        }
        self.status = if self.zen_mode.is_enabled() {
            Some("Zen Mode enabled".to_owned())
        } else {
            Some("Zen Mode disabled".to_owned())
        };
        self.save_config();
    }

    fn toggle_task_at_scroll(&mut self) {
        let Some(path) = self.active_path() else {
            self.status = Some("Task toggles require a file-backed document".to_owned());
            return;
        };
        let Some(line_index) = self.task_source_line_at_scroll() else {
            self.status = Some("No task checkbox at the reader line".to_owned());
            return;
        };
        let Some(updated_source) =
            toggle_task_line_source(self.active_document().source(), line_index)
        else {
            self.status = Some("Task checkbox source line was not found".to_owned());
            return;
        };

        if let Err(error) = fs::write(&path, updated_source) {
            self.status = Some(format!("Failed to update {}: {error}", path.display()));
            return;
        }

        self.reload_path(path);
        self.status = Some("Toggled task checkbox".to_owned());
    }

    fn task_source_line_at_scroll(&self) -> Option<usize> {
        let rendered_line = usize::from(self.scroll);
        let anchor_index = self
            .block_line_starts
            .iter()
            .rposition(|anchor| anchor.line <= rendered_line)?;
        let anchor = self.block_line_starts.get(anchor_index)?;
        let next_line = self
            .block_line_starts
            .get(anchor_index + 1)
            .map_or(self.document_lines.len(), |next| next.line);
        if rendered_line >= next_line {
            return None;
        }

        let MarkdownBlock::List { items, .. } = self
            .active_document()
            .parsed()
            .blocks
            .get(anchor.block_index)?
        else {
            return None;
        };
        let item_index = rendered_line.saturating_sub(anchor.line);

        items
            .get(item_index)
            .filter(|item| item.checked.is_some())
            .and_then(|item| item.source_line)
    }

    fn active_path(&self) -> Option<std::path::PathBuf> {
        self.documents
            .active()
            .and_then(Document::path)
            .map(std::path::PathBuf::from)
    }

    fn select_next_split_tab(&mut self) {
        self.select_split_tab_offset(1);
    }

    fn select_previous_split_tab(&mut self) {
        self.select_split_tab_offset(-1);
    }

    fn select_split_tab_offset(&mut self, offset: isize) {
        let next_index = self.split_view.cycle_secondary(
            self.documents.active_index(),
            self.documents.len(),
            offset,
        );
        if next_index == self.split_view.secondary_index() {
            return;
        }

        let Some(next_index) = next_index else {
            self.split_view.disable();
            self.split_document_lines.clear();
            self.split_scroll = 0;
            self._split_watcher = None;
            self.split_watch_receiver = None;
            self.status = Some("Split View disabled".to_owned());
            return;
        };

        self.split_view.select_secondary(
            next_index,
            self.documents.active_index(),
            self.documents.len(),
        );
        self.refresh_split_document();
        let title = self
            .documents
            .iter()
            .find_map(|(index, document)| (index == next_index).then_some(document.title()))
            .unwrap_or("Split");
        self.status = Some(format!(
            "Side {}/{}: {title}",
            next_index + 1,
            self.documents.len()
        ));
    }

    fn ensure_split_target(&mut self) {
        self.split_view
            .retarget(self.documents.active_index(), self.documents.len());
        self.refresh_split_document();
    }

    fn refresh_split_document(&mut self) {
        self.split_document_lines = self
            .split_view
            .secondary_index()
            .and_then(|split_index| {
                self.documents
                    .iter()
                    .find_map(|(index, document)| (index == split_index).then_some(document))
            })
            .map_or_else(Vec::new, |document| {
                render::render_document_with_anchors(document).lines
            });
        let (watcher, receiver, status) = self
            .split_document()
            .map_or_else(|| (None, None, None), watch_document);
        self._split_watcher = watcher;
        self.split_watch_receiver = receiver;
        if let Some(status) = status {
            self.status = Some(status);
        }
        self.sync_split_scroll();
    }

    fn split_document(&self) -> Option<&Document> {
        let split_index = self.split_view.secondary_index()?;
        self.documents
            .iter()
            .find_map(|(index, document)| (index == split_index).then_some(document))
    }

    fn split_index_for_path(&self, path: &std::path::Path) -> Option<usize> {
        let split_index = self.split_view.secondary_index()?;
        self.documents.iter().find_map(|(index, document)| {
            (index == split_index && document.path().is_some_and(|open_path| open_path == path))
                .then_some(index)
        })
    }

    fn sync_split_scroll(&mut self) {
        self.split_scroll = paperview_core::synced_scroll_offset(
            self.scroll,
            self.max_scroll(),
            self.max_split_scroll(),
        );
    }

    fn load_active_document(&mut self, reset_scroll: bool) {
        let (rendered, toc, watcher, watch_receiver, watch_status) = {
            let document = self.active_document();
            let rendered = render::render_document_with_anchors(document);
            let toc = document.parsed().toc();
            let (watcher, watch_receiver, watch_status) = watch_document(document);
            (rendered, toc, watcher, watch_receiver, watch_status)
        };

        self.document_lines = rendered.lines;
        self.block_line_starts = rendered.block_line_starts;
        self.toc = toc;
        self.toc_selected_index = if reset_scroll {
            initial_toc_selection(&self.toc)
        } else {
            clamp_toc_selection(self.toc_selected_index, self.toc.len())
        };
        self.refresh_search_matches();
        if self.toc.is_empty() {
            self.focus = ReaderFocus::Reader;
        }
        if reset_scroll {
            self.scroll = 0;
        }
        self.scroll = self.scroll.min(self.max_scroll());
        self.sync_split_scroll();

        self._watcher = watcher;
        self.watch_receiver = watch_receiver;
        if let Some(status) = watch_status {
            self.status = Some(status);
        }
        self.ensure_split_target();
    }

    fn toggle_focus(&mut self) {
        if self.zen_mode.is_enabled() {
            self.focus = ReaderFocus::Reader;
            return;
        }

        if self.toc.is_empty() {
            self.focus = ReaderFocus::Reader;
            self.toc_selected_index = None;
            return;
        }

        self.focus = match self.focus {
            ReaderFocus::Reader => ReaderFocus::Toc,
            ReaderFocus::Toc => ReaderFocus::Reader,
        };

        if self.focus == ReaderFocus::Toc && self.toc_selected_index.is_none() {
            self.toc_selected_index = self
                .active_toc_block_index()
                .and_then(|block_index| self.toc_index_for_block(block_index))
                .or(Some(0));
        }
    }

    fn select_next_toc(&mut self) {
        if self.toc.is_empty() {
            self.toc_selected_index = None;
            return;
        }

        let next = self
            .toc_selected_index
            .map_or(0, |index| (index + 1).min(self.toc.len() - 1));
        self.toc_selected_index = Some(next);
    }

    fn select_previous_toc(&mut self) {
        if self.toc.is_empty() {
            self.toc_selected_index = None;
            return;
        }

        let previous = self
            .toc_selected_index
            .map_or(0, |index| index.saturating_sub(1));
        self.toc_selected_index = Some(previous);
    }

    fn jump_to_selected_toc(&mut self) {
        if self.focus != ReaderFocus::Toc {
            return;
        }

        let Some(index) = self.toc_selected_index else {
            return;
        };
        let Some(item) = self.toc.get(index) else {
            return;
        };

        self.scroll = self
            .block_line_start(item.block_index)
            .min(usize::from(self.max_scroll())) as u16;
        self.sync_split_scroll();
    }

    fn active_toc_block_index(&self) -> Option<usize> {
        let target_line = usize::from(self.scroll);

        self.toc
            .iter()
            .rfind(|item| self.block_line_start(item.block_index) <= target_line)
            .or_else(|| self.toc.first())
            .map(|item| item.block_index)
    }

    fn block_line_start(&self, block_index: usize) -> usize {
        self.block_line_starts
            .iter()
            .find(|anchor| anchor.block_index == block_index)
            .map_or(usize::MAX, |anchor| anchor.line)
    }

    fn toc_index_for_block(&self, block_index: usize) -> Option<usize> {
        self.toc
            .iter()
            .position(|item| item.block_index == block_index)
    }

    fn max_scroll(&self) -> u16 {
        self.document_lines.len().saturating_sub(1) as u16
    }

    fn max_split_scroll(&self) -> u16 {
        self.split_document_lines.len().saturating_sub(1) as u16
    }

    fn handle_watch_events(&mut self) {
        if let Some(receiver) = self.watch_receiver.take() {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    WatchEvent::Changed(path) => self.reload_path(path),
                }
            }

            self.watch_receiver = Some(receiver);
        }

        if let Some(receiver) = self.split_watch_receiver.take() {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    WatchEvent::Changed(path) => self.reload_path(path),
                }
            }

            self.split_watch_receiver = Some(receiver);
        }
    }

    fn reload_path(&mut self, path: std::path::PathBuf) {
        if self.active_document().path() == Some(&path) {
            match Document::open(&path) {
                Ok(document) => {
                    self.documents.replace_active(document);
                    self.load_active_document(false);
                    self.status = Some(format!("Reloaded {}", path.display()));
                }
                Err(error) => {
                    self.status = Some(error.to_string());
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
                self.refresh_split_document();
                self.status = Some(format!("Reloaded side {}", path.display()));
            }
            Err(error) => {
                self.status = Some(error.to_string());
            }
        }
    }

    fn save_config(&mut self) {
        self.config.zen_mode = self.zen_mode.is_enabled();
        self.config.split_primary_width = self.split_view.primary_width();
        if let Err(error) = self.config_store.save(&self.config) {
            self.status = Some(error.to_string());
        }
    }
}

fn load_config(store: &ConfigStore) -> (Config, Option<String>) {
    match store.load() {
        Ok(config) => (config, None),
        Err(error) => (Config::default(), Some(error.to_string())),
    }
}

#[cfg(not(test))]
fn default_config_store() -> ConfigStore {
    ConfigStore::default()
}

#[cfg(test)]
fn default_config_store() -> ConfigStore {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();

    ConfigStore::new(std::env::temp_dir().join(format!("paperview-tui-{nanos}-config.toml")))
}

#[cfg(not(test))]
fn default_history_store() -> HistoryStore {
    HistoryStore::default()
}

#[cfg(test)]
fn default_history_store() -> HistoryStore {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();

    HistoryStore::new(std::env::temp_dir().join(format!("paperview-tui-{nanos}-history.json")))
}

#[derive(Debug)]
struct WorkspaceSearchApp {
    query: String,
    root: std::path::PathBuf,
    matches: Vec<WorkspaceSearchMatch>,
    list_state: ListState,
    store: HistoryStore,
    status: Option<String>,
}

#[derive(Debug)]
struct BookmarkApp {
    bookmarks: Bookmarks,
    list_state: ListState,
    bookmark_store: BookmarkStore,
    history_store: HistoryStore,
    status: Option<String>,
}

impl BookmarkApp {
    fn new(bookmark_store: BookmarkStore, history_store: HistoryStore) -> Self {
        let mut status = None;
        let mut bookmarks = bookmark_store.load().unwrap_or_else(|error| {
            status = Some(error.to_string());
            Bookmarks::new()
        });
        let pruned = bookmarks.prune_missing();
        if pruned > 0
            && let Err(error) = bookmark_store.save(&bookmarks)
        {
            status = Some(error.to_string());
        }
        let mut list_state = ListState::default();
        if !bookmarks.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            bookmarks,
            list_state,
            bookmark_store,
            history_store,
            status,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Char('p') => self.prune_missing(),
                    KeyCode::Enter => {
                        if let Some((document, line_number)) = self.open_selected() {
                            ReaderApp::new_at_source_line_with_status(
                                document,
                                line_number,
                                format!("Opened bookmark at line {line_number}"),
                            )
                            .run(terminal)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        frame.render_widget(
            Paragraph::new(" PaperView - Bookmarks ")
                .style(theme::shell())
                .block(Block::default().borders(Borders::BOTTOM)),
            header,
        );

        if self.bookmarks.is_empty() {
            frame.render_widget(
                Paragraph::new("No bookmarks.")
                    .block(Block::default().title("Bookmarks").borders(Borders::ALL))
                    .style(theme::reader())
                    .wrap(Wrap { trim: true }),
                body,
            );
        } else {
            let items = self
                .bookmarks
                .entries()
                .iter()
                .map(bookmark_item)
                .collect::<Vec<_>>();
            let list = List::new(items)
                .block(Block::default().title("Bookmarks").borders(Borders::ALL))
                .highlight_symbol("> ")
                .highlight_style(theme::list_highlight());

            frame.render_stateful_widget(list, body, &mut self.list_state);
        }

        let status = self
            .status
            .as_deref()
            .unwrap_or("Enter opens selected bookmark - j/k move - p prunes - q quits");
        frame.render_widget(Paragraph::new(status).style(theme::status()), footer);
    }

    fn select_next(&mut self) {
        let len = self.bookmarks.entries().len();
        if len == 0 {
            return;
        }

        let next = self
            .list_state
            .selected()
            .map_or(0, |index| (index + 1).min(len - 1));
        self.list_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        let len = self.bookmarks.entries().len();
        if len == 0 {
            return;
        }

        let previous = self
            .list_state
            .selected()
            .map_or(0, |index| index.saturating_sub(1));
        self.list_state.select(Some(previous));
    }

    fn prune_missing(&mut self) {
        let removed = self.bookmarks.prune_missing();
        if let Err(error) = self.bookmark_store.save(&self.bookmarks) {
            self.status = Some(error.to_string());
        } else {
            self.status = Some(format!("Pruned {removed} missing bookmark(s)"));
        }
        self.list_state.select(clamp_bookmark_selection(
            self.list_state.selected(),
            self.bookmarks.entries().len(),
        ));
    }

    fn open_selected(&mut self) -> Option<(Document, usize)> {
        let index = self.list_state.selected()?;
        let bookmark = self.bookmarks.entries().get(index)?;
        let path = bookmark.path().to_path_buf();
        let line_number = bookmark.source_line().unwrap_or(1);

        match Document::open(&path) {
            Ok(document) => {
                self.record_opened_document(&document);
                self.status = None;
                Some((document, line_number))
            }
            Err(error) => {
                self.status = Some(error.to_string());
                None
            }
        }
    }

    fn record_opened_document(&mut self, document: &Document) {
        let mut history = self.history_store.load().unwrap_or_else(|error| {
            self.status = Some(error.to_string());
            History::new()
        });
        history.record_document(document);
        if let Err(error) = self.history_store.save(&history) {
            self.status = Some(error.to_string());
        }
    }
}

impl WorkspaceSearchApp {
    fn new(
        query: String,
        root: std::path::PathBuf,
        matches: Vec<WorkspaceSearchMatch>,
        store: HistoryStore,
    ) -> Self {
        let mut list_state = ListState::default();
        if !matches.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            query,
            root,
            matches,
            list_state,
            store,
            status: None,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Enter => {
                        if let Some((document, line_number)) = self.open_selected() {
                            ReaderApp::new_at_source_line(document, line_number).run(terminal)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        frame.render_widget(
            Paragraph::new(format!(
                " PaperView - Search '{}' in {} ",
                self.query,
                self.root.display()
            ))
            .style(theme::shell())
            .block(Block::default().borders(Borders::BOTTOM)),
            header,
        );

        if self.matches.is_empty() {
            frame.render_widget(
                Paragraph::new("No matches.")
                    .block(
                        Block::default()
                            .title("Workspace Search")
                            .borders(Borders::ALL),
                    )
                    .style(theme::reader())
                    .wrap(Wrap { trim: true }),
                body,
            );
        } else {
            let items = self
                .matches
                .iter()
                .map(workspace_search_item)
                .collect::<Vec<_>>();
            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Workspace Search")
                        .borders(Borders::ALL),
                )
                .highlight_symbol("> ")
                .highlight_style(theme::list_highlight());

            frame.render_stateful_widget(list, body, &mut self.list_state);
        }

        let status = self
            .status
            .as_deref()
            .unwrap_or("Enter opens selected match - j/k move - q quits");
        frame.render_widget(Paragraph::new(status).style(theme::status()), footer);
    }

    fn select_next(&mut self) {
        let len = self.matches.len();
        if len == 0 {
            return;
        }

        let next = self
            .list_state
            .selected()
            .map_or(0, |index| (index + 1).min(len - 1));
        self.list_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        let len = self.matches.len();
        if len == 0 {
            return;
        }

        let previous = self
            .list_state
            .selected()
            .map_or(0, |index| index.saturating_sub(1));
        self.list_state.select(Some(previous));
    }

    fn open_selected(&mut self) -> Option<(Document, usize)> {
        let index = self.list_state.selected()?;
        let search_match = self.matches.get(index)?;
        let path = search_match.path.clone();
        let line_number = search_match.line_number;

        match Document::open(&path) {
            Ok(document) => {
                self.record_opened_document(&document);
                self.status = None;
                Some((document, line_number))
            }
            Err(error) => {
                self.status = Some(error.to_string());
                None
            }
        }
    }

    fn record_opened_document(&mut self, document: &Document) {
        let mut history = self.store.load().unwrap_or_else(|error| {
            self.status = Some(error.to_string());
            History::new()
        });
        history.record_document(document);
        if let Err(error) = self.store.save(&history) {
            self.status = Some(error.to_string());
        }
    }
}

fn initial_toc_selection(toc: &[TocItem]) -> Option<usize> {
    (!toc.is_empty()).then_some(0)
}

fn clamp_toc_selection(selection: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(selection.unwrap_or(0).min(len - 1))
    }
}

fn clamp_search_selection(selection: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(selection.unwrap_or(0).min(len - 1))
    }
}

fn clamp_bookmark_selection(selection: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(selection.unwrap_or(0).min(len - 1))
    }
}

fn tab_line(documents: &OpenDocuments) -> Line<'static> {
    let mut spans = Vec::new();

    for (index, document) in documents.iter() {
        if index > 0 {
            spans.push(Span::styled(" ", theme::shell()));
        }
        let is_active = documents.active_index() == Some(index);
        let style = if is_active {
            theme::tab_active()
        } else {
            theme::tab_inactive()
        };
        spans.push(Span::styled(
            format!(" {}:{} ", index + 1, document.title()),
            style,
        ));
    }

    Line::from(spans)
}

fn watch_document(
    document: &Document,
) -> (
    Option<FileWatcher>,
    Option<Receiver<WatchEvent>>,
    Option<String>,
) {
    let Some(path) = document.path() else {
        return (None, None, None);
    };
    let (sender, receiver) = mpsc::channel();

    match watch_file(path, sender) {
        Ok(watcher) => (Some(watcher), Some(receiver), None),
        Err(error) => (None, None, Some(error.to_string())),
    }
}

#[derive(Debug)]
struct DashboardApp {
    history: History,
    store: HistoryStore,
    list_state: ListState,
    status: Option<String>,
}

impl DashboardApp {
    fn new(store: HistoryStore) -> Self {
        let history = load_pruned_history(&store);
        let mut list_state = ListState::default();

        if !history.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            history,
            store,
            list_state,
            status: None,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                    KeyCode::Enter => {
                        if let Some(document) = self.open_selected() {
                            ReaderApp::new(document).run(terminal)?;
                            self.history = load_pruned_history(&self.store);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());

        frame.render_widget(
            Paragraph::new(" PaperView - Recent files ")
                .style(theme::shell())
                .block(Block::default().borders(Borders::BOTTOM)),
            header,
        );

        if self.history.is_empty() {
            frame.render_widget(
                Paragraph::new("No recent files yet.\n\nOpen a file with paperview-tui <file>.")
                    .block(Block::default().title("History").borders(Borders::ALL))
                    .style(theme::reader())
                    .wrap(Wrap { trim: true }),
                body,
            );
        } else {
            let items = self
                .history
                .entries()
                .iter()
                .map(history_item)
                .collect::<Vec<_>>();
            let list = List::new(items)
                .block(Block::default().title("History").borders(Borders::ALL))
                .highlight_symbol("> ")
                .highlight_style(theme::list_highlight());

            frame.render_stateful_widget(list, body, &mut self.list_state);
        }

        let status = self
            .status
            .as_deref()
            .unwrap_or("Enter opens selected file - j/k move - q quits");
        frame.render_widget(Paragraph::new(status).style(theme::status()), footer);
    }

    fn select_next(&mut self) {
        let len = self.history.entries().len();
        if len == 0 {
            return;
        }

        let next = self
            .list_state
            .selected()
            .map_or(0, |index| (index + 1).min(len - 1));
        self.list_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        let len = self.history.entries().len();
        if len == 0 {
            return;
        }

        let previous = self
            .list_state
            .selected()
            .map_or(0, |index| index.saturating_sub(1));
        self.list_state.select(Some(previous));
    }

    fn open_selected(&mut self) -> Option<Document> {
        let index = self.list_state.selected()?;
        let entry = self.history.entries().get(index)?;
        let path = entry.path().to_path_buf();

        match Document::open(&path) {
            Ok(document) => {
                self.record_opened_document(&document);
                self.status = None;
                Some(document)
            }
            Err(error) => {
                self.status = Some(error.to_string());
                None
            }
        }
    }

    fn record_opened_document(&mut self, document: &Document) {
        self.history.record_document(document);
        if let Err(error) = self.store.save(&self.history) {
            self.status = Some(error.to_string());
        }
    }
}

fn load_pruned_history(store: &HistoryStore) -> History {
    let mut history = store.load().unwrap_or_else(|error| {
        eprintln!("{error}");
        History::new()
    });
    if history.prune_missing() > 0
        && let Err(error) = store.save(&history)
    {
        eprintln!("{error}");
    }
    history
}

fn history_item(entry: &FileEntry) -> ListItem<'static> {
    ListItem::new(vec![
        Line::from(Span::styled(entry.title().to_owned(), theme::list_title())),
        Line::from(Span::styled(
            entry.path().display().to_string(),
            theme::list_meta(),
        )),
    ])
}

fn bookmark_item(bookmark: &Bookmark) -> ListItem<'static> {
    let mut meta = bookmark.path().display().to_string();
    if let Some(anchor) = bookmark.heading_anchor() {
        meta.push_str(&format!(" #{anchor}"));
    }
    if let Some(line) = bookmark.source_line() {
        meta.push_str(&format!(" line {line}"));
    }

    ListItem::new(vec![
        Line::from(Span::styled(
            bookmark.title().to_owned(),
            theme::list_title(),
        )),
        Line::from(Span::styled(meta, theme::list_meta())),
    ])
}

fn workspace_search_item(search_match: &WorkspaceSearchMatch) -> ListItem<'static> {
    ListItem::new(vec![
        Line::from(Span::styled(
            format!(
                "{}:{}:{}",
                search_match.path.display(),
                search_match.line_number,
                search_match.column
            ),
            theme::list_title(),
        )),
        Line::from(Span::styled(
            search_match.line.trim().to_owned(),
            theme::list_meta(),
        )),
    ])
}

#[derive(Debug, Clone, Copy, Default)]
struct SearchHighlights<'a> {
    query: &'a str,
    matches: &'a [SearchMatch],
    selected_index: Option<usize>,
}

impl SearchHighlights<'_> {
    fn line_state(self, line_index: usize) -> SearchLineState {
        if self
            .selected_index
            .and_then(|index| self.matches.get(index))
            .is_some_and(|search_match| search_match.line_index == line_index)
        {
            return SearchLineState::Selected;
        }

        if self
            .matches
            .iter()
            .any(|search_match| search_match.line_index == line_index)
        {
            SearchLineState::Matched
        } else {
            SearchLineState::None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchLineState {
    None,
    Matched,
    Selected,
}

fn document_text(lines: &[String], highlights: SearchHighlights<'_>) -> Vec<Line<'static>> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| document_line(line, highlights.query, highlights.line_state(index)))
        .collect()
}

fn edit_buffer_text(buffer: &str, cursor: usize) -> Vec<Line<'static>> {
    if buffer.is_empty() {
        return vec![Line::from(Span::styled(
            " ",
            theme::reader().add_modifier(Modifier::REVERSED),
        ))];
    }

    let cursor = clamp_to_char_boundary(buffer, cursor);
    let (cursor_line, cursor_column) = cursor_line_column(buffer, cursor);
    buffer
        .split('\n')
        .enumerate()
        .map(|(line_index, line)| {
            if line_index == cursor_line {
                edit_cursor_line(line, cursor_column)
            } else {
                Line::from(line.to_owned())
            }
        })
        .collect()
}

fn edit_cursor_line(line: &str, cursor_column: usize) -> Line<'static> {
    let cursor_byte = line_byte_for_column(line, cursor_column);
    let (before, rest) = line.split_at(cursor_byte);
    let mut spans = vec![Span::raw(before.to_owned())];
    if let Some(character) = rest.chars().next() {
        let character_len = character.len_utf8();
        spans.push(Span::styled(
            character.to_string(),
            theme::reader().add_modifier(Modifier::REVERSED),
        ));
        spans.push(Span::raw(rest[character_len..].to_owned()));
    } else {
        spans.push(Span::styled(
            " ",
            theme::reader().add_modifier(Modifier::REVERSED),
        ));
    }
    Line::from(spans)
}

fn previous_char_boundary(buffer: &str, cursor: usize) -> Option<usize> {
    let cursor = clamp_to_char_boundary(buffer, cursor);
    (cursor > 0).then(|| {
        buffer[..cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index)
    })
}

fn next_char_boundary(buffer: &str, cursor: usize) -> Option<usize> {
    let cursor = clamp_to_char_boundary(buffer, cursor);
    buffer[cursor..]
        .chars()
        .next()
        .map(|character| cursor + character.len_utf8())
}

fn clamp_to_char_boundary(buffer: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(buffer.len());
    while cursor > 0 && !buffer.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn cursor_line_column(buffer: &str, cursor: usize) -> (usize, usize) {
    let cursor = clamp_to_char_boundary(buffer, cursor);
    let mut line = 0;
    let mut column = 0;
    for character in buffer[..cursor].chars() {
        if character == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn cursor_offset_for_line_column(buffer: &str, target_line: usize, target_column: usize) -> usize {
    let mut line = 0;
    let mut column = 0;
    for (index, character) in buffer.char_indices() {
        if line == target_line && column == target_column {
            return index;
        }
        if line == target_line && character == '\n' {
            return index;
        }
        if character == '\n' {
            line += 1;
            column = 0;
            if line > target_line {
                return index;
            }
        } else {
            column += 1;
        }
    }
    buffer.len()
}

fn cursor_offset_for_line_end(buffer: &str, target_line: usize) -> usize {
    let mut line = 0;
    for (index, character) in buffer.char_indices() {
        if line == target_line && character == '\n' {
            return index;
        }
        if character == '\n' {
            line += 1;
        }
    }
    buffer.len()
}

fn line_byte_for_column(line: &str, target_column: usize) -> usize {
    line.char_indices()
        .nth(target_column)
        .map_or(line.len(), |(index, _)| index)
}

fn edit_line_count(buffer: &str) -> usize {
    buffer.split('\n').count().max(1)
}

fn edit_max_scroll(buffer: &str, viewport_lines: usize) -> u16 {
    edit_line_count(buffer)
        .saturating_sub(viewport_lines.max(1))
        .min(usize::from(u16::MAX)) as u16
}

fn edit_preview_max_scroll(lines: &[String], viewport_lines: usize) -> u16 {
    lines
        .len()
        .saturating_sub(viewport_lines.max(1))
        .min(usize::from(u16::MAX)) as u16
}

fn offset_scroll(current: u16, delta: isize, max_scroll: u16) -> u16 {
    if delta.is_negative() {
        current.saturating_sub(delta.unsigned_abs() as u16)
    } else {
        current.saturating_add(delta as u16).min(max_scroll)
    }
}

fn document_line(line: &str, query: &str, search_state: SearchLineState) -> Line<'static> {
    if search_state != SearchLineState::None {
        return highlighted_document_line(line, query, search_state);
    }

    if line.starts_with('#') {
        Line::from(Span::styled(line.to_owned(), theme::reader_heading()))
    } else if line.starts_with("> ") {
        Line::from(Span::styled(line.to_owned(), theme::reader_quote()))
    } else {
        Line::from(line.to_owned())
    }
}

fn highlighted_document_line(
    line: &str,
    query: &str,
    search_state: SearchLineState,
) -> Line<'static> {
    let base_style = match search_state {
        SearchLineState::Selected => theme::search_selected(),
        SearchLineState::Matched => theme::search_matched(),
        SearchLineState::None => Style::default(),
    };
    let emphasis_style = match search_state {
        SearchLineState::Selected => theme::search_selected_emphasis(),
        SearchLineState::Matched => theme::search_matched_emphasis(),
        SearchLineState::None => Style::default(),
    };
    let Some((start, end)) = match_range(line, query) else {
        return Line::from(Span::styled(line.to_owned(), base_style));
    };

    Line::from(vec![
        Span::styled(line[..start].to_owned(), base_style),
        Span::styled(line[start..end].to_owned(), emphasis_style),
        Span::styled(line[end..].to_owned(), base_style),
    ])
}

fn match_range(line: &str, query: &str) -> Option<(usize, usize)> {
    let needle = query.trim();
    if needle.is_empty() {
        return None;
    }

    let lower_line = line.to_lowercase();
    let lower_needle = needle.to_lowercase();
    let start = lower_line.find(&lower_needle)?;
    Some((start, start + lower_needle.len()))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use paperview_core::{
        Config, ConfigStore, Document, FileEntry, HistoryStore, SearchMatch, ThemePreference,
        WorkspaceSearchMatch,
    };
    use ratatui::style::Modifier;

    use super::{
        BookmarkApp, DashboardApp, EDIT_VIEWPORT_LINES, EditMode, OpenPathMode, PresentationMode,
        ReaderApp, ReaderFocus, SearchHighlights, SearchMode, WorkspaceSearchApp,
        clamp_bookmark_selection, clamp_search_selection, clamp_toc_selection, cursor_line_column,
        document_text, edit_preview_max_scroll, tab_line,
    };
    use crate::theme;

    #[test]
    fn scrolling_is_saturating() {
        let mut app = ReaderApp::new(Document::from_source("# Title\n\nBody"));

        app.scroll_up();
        assert_eq!(app.scroll, 0);

        app.scroll_to_bottom();
        let bottom = app.scroll;
        app.scroll_down();
        assert_eq!(app.scroll, bottom);
    }

    #[test]
    fn active_toc_tracks_scroll_position() {
        let mut app = ReaderApp::new(Document::from_source(
            "# First\n\nOne.\n\n## Second\n\nTwo.\n\n## Third\n\nThree.",
        ));

        assert_eq!(app.active_toc_block_index(), Some(0));

        app.scroll = app.block_line_start(2) as u16;
        assert_eq!(app.active_toc_block_index(), Some(2));

        app.scroll = app.block_line_start(4) as u16;
        assert_eq!(app.active_toc_block_index(), Some(4));
    }

    #[test]
    fn active_toc_falls_back_to_first_heading() {
        let app = ReaderApp::new(Document::from_source("# First\n\nBody."));

        assert_eq!(app.active_toc_block_index(), Some(0));
    }

    #[test]
    fn active_toc_is_empty_without_headings() {
        let app = ReaderApp::new(Document::from_source("Body only."));

        assert_eq!(app.active_toc_block_index(), None);
    }

    #[test]
    fn toc_focus_toggle_requires_headings() {
        let mut app = ReaderApp::new(Document::from_source("# First\n\nBody."));

        assert_eq!(app.focus, ReaderFocus::Reader);
        app.toggle_focus();
        assert_eq!(app.focus, ReaderFocus::Toc);
        app.toggle_focus();
        assert_eq!(app.focus, ReaderFocus::Reader);

        let mut empty = ReaderApp::new(Document::from_source("Body only."));
        empty.toggle_focus();
        assert_eq!(empty.focus, ReaderFocus::Reader);
        assert_eq!(empty.toc_selected_index, None);
    }

    #[test]
    fn zen_toggle_forces_reader_focus_and_blocks_toc_focus() {
        let mut app = ReaderApp::new(Document::from_source("# First\n\nBody."));
        app.toggle_focus();
        assert_eq!(app.focus, ReaderFocus::Toc);

        app.toggle_zen();
        assert!(app.zen_mode.is_enabled());
        assert_eq!(app.focus, ReaderFocus::Reader);

        app.toggle_focus();
        assert_eq!(app.focus, ReaderFocus::Reader);

        app.toggle_zen();
        assert!(!app.zen_mode.is_enabled());
        app.toggle_focus();
        assert_eq!(app.focus, ReaderFocus::Toc);
    }

    #[test]
    fn tui_loads_zen_and_split_width_from_config() {
        let path = temp_doc("tui-config-load.toml", "");
        let store = ConfigStore::new(&path);
        store
            .save(&Config {
                schema_version: 1,
                theme: ThemePreference::Hybrid,
                zen_mode: true,
                split_primary_width: 65,
                tex_compiler_path: None,
            })
            .expect("save config");

        let app = ReaderApp::new_documents_with_config(
            vec![Document::from_source("# First").with_path("first.md")],
            store,
        );

        assert!(app.zen_mode.is_enabled());
        assert_eq!(app.split_widths(), (65, 35));
        assert_eq!(app.config.theme, ThemePreference::Hybrid);

        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn tui_persists_zen_and_split_width_to_config() {
        let path = temp_doc("tui-config-save.toml", "");
        let store = ConfigStore::new(&path);
        store.ensure_exists().expect("ensure config");
        let mut app = ReaderApp::new_documents_with_config(
            vec![
                Document::from_source("# First").with_path("first.md"),
                Document::from_source("# Second").with_path("second.md"),
            ],
            store.clone(),
        );

        app.toggle_split();
        app.grow_split_primary();
        app.toggle_zen();

        let config = store.load().expect("load saved config");
        assert!(config.zen_mode);
        assert_eq!(config.split_primary_width, 60);

        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn zen_header_replaces_tab_line() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
        ]);

        assert!(app.header_lines()[1].spans[0].content.contains("1:First"));

        app.toggle_zen();

        assert_eq!(
            app.header_lines()[1].spans[0].content.as_ref(),
            " Zen Mode "
        );
    }

    #[test]
    fn task_toggle_updates_file_backed_active_document() {
        let path = temp_doc("tui-task-toggle.md", "# Tasks\n\n- [ ] Todo\n- [x] Done");
        let document = Document::open(&path).expect("open task document");
        let mut app = ReaderApp::new(document);

        app.scroll = 2;
        assert_eq!(app.task_source_line_at_scroll(), Some(2));

        app.toggle_task_at_scroll();

        let updated = fs::read_to_string(&path).expect("read updated task document");
        assert_eq!(updated, "# Tasks\n\n- [x] Todo\n- [x] Done");
        assert!(app.document_lines.iter().any(|line| line == "- [x] Todo"));
        assert_eq!(app.status.as_deref(), Some("Toggled task checkbox"));

        fs::remove_file(path).expect("remove task document");
    }

    #[test]
    fn task_toggle_requires_file_backed_task_line() {
        let mut app = ReaderApp::new(Document::from_source("# Tasks\n\n- [ ] Todo"));
        app.scroll = 2;

        app.toggle_task_at_scroll();

        assert_eq!(
            app.status.as_deref(),
            Some("Task toggles require a file-backed document")
        );

        let path = temp_doc("tui-task-no-checkbox.md", "# Tasks\n\nBody");
        let document = Document::open(&path).expect("open non-task document");
        let mut app = ReaderApp::new(document);
        app.scroll = 2;

        app.toggle_task_at_scroll();

        assert_eq!(
            app.status.as_deref(),
            Some("No task checkbox at the reader line")
        );

        fs::remove_file(path).expect("remove non-task document");
    }

    #[test]
    fn toc_selection_is_bounded() {
        let mut app = ReaderApp::new(Document::from_source("# First\n\n## Second"));
        app.toggle_focus();

        app.select_previous_toc();
        assert_eq!(app.toc_selected_index, Some(0));

        app.select_next_toc();
        app.select_next_toc();
        assert_eq!(app.toc_selected_index, Some(1));
    }

    #[test]
    fn toc_jump_scrolls_to_selected_heading() {
        let mut app = ReaderApp::new(Document::from_source("# First\n\nBody.\n\n## Second"));
        app.toggle_focus();
        app.select_next_toc();
        app.jump_to_selected_toc();

        assert_eq!(app.scroll, app.block_line_start(2) as u16);
        assert_eq!(app.active_toc_block_index(), Some(2));
    }

    #[test]
    fn tab_navigation_wraps_between_documents() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second\n\n## Two").with_path("second.md"),
        ]);

        assert_eq!(app.active_document().title(), "First");

        app.select_next_tab();
        assert_eq!(app.active_document().title(), "Second");
        assert_eq!(app.documents.active_index(), Some(1));
        assert_eq!(app.toc.len(), 2);
        assert_eq!(app.scroll, 0);

        app.select_next_tab();
        assert_eq!(app.active_document().title(), "First");

        app.select_previous_tab();
        assert_eq!(app.active_document().title(), "Second");
    }

    #[test]
    fn open_path_prompt_opens_file_as_active_tab() {
        let path = temp_doc("tui-open-path.md", "# Opened\n\nFrom prompt.");
        let mut app = ReaderApp::new(Document::from_source("# First").with_path("first.md"));

        app.start_open_path();
        for character in path.display().to_string().chars() {
            app.handle_open_path_key(KeyCode::Char(character));
        }
        app.handle_open_path_key(KeyCode::Enter);

        assert_eq!(app.open_path_mode, OpenPathMode::Inactive);
        assert_eq!(app.documents.len(), 2);
        assert_eq!(app.active_document().title(), "Opened");
        assert_eq!(app.scroll, 0);
        assert!(
            app.status
                .as_deref()
                .is_some_and(|status| status.starts_with("Opened tab 2/2: Opened"))
        );
        assert!(app.document_lines.iter().any(|line| line == "# Opened"));

        fs::remove_file(path).expect("remove opened document");
    }

    #[test]
    fn open_path_prompt_reports_empty_and_invalid_paths() {
        let mut app = ReaderApp::new(Document::from_source("# First").with_path("first.md"));

        app.start_open_path();
        app.handle_open_path_key(KeyCode::Enter);

        assert_eq!(app.open_path_mode, OpenPathMode::Inactive);
        assert_eq!(app.status.as_deref(), Some("Open path is empty"));
        assert_eq!(app.documents.len(), 1);

        let unsupported_path = temp_doc("tui-open-path.png", "not markdown");
        app.start_open_path();
        for character in unsupported_path.display().to_string().chars() {
            app.handle_open_path_key(KeyCode::Char(character));
        }
        app.handle_open_path_key(KeyCode::Enter);

        assert_eq!(app.documents.len(), 1);
        assert!(
            app.status
                .as_deref()
                .is_some_and(|status| status.contains("unsupported document type"))
        );

        fs::remove_file(unsupported_path).expect("remove unsupported document");
    }

    #[test]
    fn split_toggle_targets_first_non_active_tab() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second\n\nSide\n\nMore").with_path("second.md"),
        ]);

        app.toggle_split();

        assert_eq!(app.split_view.secondary_index(), Some(1));
        assert!(
            app.split_document_lines
                .iter()
                .any(|line| line == "# Second")
        );

        app.toggle_split();

        assert_eq!(app.split_view.secondary_index(), None);
        assert!(app.split_document_lines.is_empty());
        assert_eq!(app.split_scroll, 0);
    }

    #[test]
    fn split_scroll_tracks_primary_reader_progress() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First\n\nOne\n\nTwo\n\nThree\n\nFour").with_path("first.md"),
            Document::from_source("# Second\n\nA\n\nB").with_path("second.md"),
        ]);
        app.toggle_split();

        app.scroll_to_bottom();

        assert_eq!(app.scroll, app.max_scroll());
        assert_eq!(app.split_scroll, app.max_split_scroll());

        app.scroll_up();

        assert!(app.split_scroll <= app.max_split_scroll());
    }

    #[test]
    fn split_toggle_requires_secondary_tab() {
        let mut app = ReaderApp::new(Document::from_source("# Only").with_path("only.md"));

        app.toggle_split();

        assert_eq!(app.split_view.secondary_index(), None);
        assert!(app.split_document_lines.is_empty());
    }

    #[test]
    fn split_retargets_when_active_tab_changes() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
            Document::from_source("# Third").with_path("third.md"),
        ]);
        app.toggle_split();

        app.select_next_tab();

        assert_ne!(
            app.split_view.secondary_index(),
            app.documents.active_index()
        );
        assert_eq!(app.split_view.secondary_index(), Some(0));
        assert!(
            app.split_document_lines
                .iter()
                .any(|line| line == "# First")
        );
    }

    #[test]
    fn split_side_navigation_wraps_through_non_active_tabs() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
            Document::from_source("# Third").with_path("third.md"),
        ]);
        app.toggle_split();

        app.select_next_split_tab();
        assert_eq!(app.split_view.secondary_index(), Some(2));
        assert!(
            app.split_document_lines
                .iter()
                .any(|line| line == "# Third")
        );

        app.select_next_split_tab();
        assert_eq!(app.split_view.secondary_index(), Some(1));

        app.select_previous_split_tab();
        assert_eq!(app.split_view.secondary_index(), Some(2));
    }

    #[test]
    fn split_side_navigation_requires_enabled_split() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
        ]);

        app.select_next_split_tab();
        assert_eq!(app.split_view.secondary_index(), None);
        assert!(app.split_document_lines.is_empty());
    }

    #[test]
    fn split_resize_changes_primary_width_when_enabled() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
        ]);
        app.toggle_split();

        app.grow_split_primary();
        assert_eq!(app.split_widths(), (60, 40));

        app.shrink_split_primary();
        app.shrink_split_primary();
        assert_eq!(app.split_widths(), (40, 60));
        assert_eq!(app.status.as_deref(), Some("Split View 40/60"));
    }

    #[test]
    fn split_resize_is_bounded_and_requires_split() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
        ]);

        app.grow_split_primary();
        assert_eq!(app.split_widths(), (50, 50));

        app.toggle_split();
        for _ in 0..8 {
            app.grow_split_primary();
        }
        assert_eq!(app.split_widths(), (70, 30));

        for _ in 0..8 {
            app.shrink_split_primary();
        }
        assert_eq!(app.split_widths(), (30, 70));
    }

    #[test]
    fn closing_split_tab_retargets_or_disables_split() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
        ]);
        app.toggle_split();
        app.select_next_tab();

        assert!(!app.close_active_tab());

        assert_eq!(app.split_view.secondary_index(), None);
        assert!(app.split_document_lines.is_empty());
    }

    #[test]
    fn tab_line_marks_active_document() {
        let app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second").with_path("second.md"),
        ]);
        let line = tab_line(&app.documents);

        assert_eq!(line.spans[0].content.as_ref(), " 1:First ");
        assert_eq!(line.spans[0].style, theme::tab_active());
        assert_eq!(line.spans[2].content.as_ref(), " 2:Second ");
        assert_eq!(line.spans[2].style, theme::tab_inactive());
    }

    #[test]
    fn closing_active_tab_selects_neighbor() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second\n\n## Two").with_path("second.md"),
            Document::from_source("# Third").with_path("third.md"),
        ]);
        app.select_next_tab();

        assert!(!app.close_active_tab());

        assert_eq!(app.documents.len(), 2);
        assert_eq!(app.active_document().title(), "Third");
        assert_eq!(app.documents.active_index(), Some(1));
        assert_eq!(app.scroll, 0);
        assert_eq!(app.status.as_deref(), Some("Closed Second"));
    }

    #[test]
    fn closing_last_tab_requests_exit() {
        let mut app = ReaderApp::new(Document::from_source("# Only").with_path("only.md"));

        assert!(app.close_active_tab());
        assert!(app.documents.is_empty());
    }

    #[test]
    fn toc_selection_clamps_after_reload() {
        assert_eq!(clamp_toc_selection(Some(3), 2), Some(1));
        assert_eq!(clamp_toc_selection(Some(0), 0), None);
        assert_eq!(clamp_toc_selection(None, 2), Some(0));
    }

    #[test]
    fn search_submission_scrolls_to_first_match() {
        let mut app = ReaderApp::new(Document::from_source("# Title\n\nAlpha.\n\nNeedle here."));

        app.start_search();
        app.handle_search_key(KeyCode::Char('n'));
        app.handle_search_key(KeyCode::Char('e'));
        app.handle_search_key(KeyCode::Char('e'));
        app.handle_search_key(KeyCode::Char('d'));
        app.handle_search_key(KeyCode::Char('l'));
        app.handle_search_key(KeyCode::Char('e'));
        app.handle_search_key(KeyCode::Enter);

        assert_eq!(app.search_mode, SearchMode::Inactive);
        assert_eq!(app.search_matches.len(), 1);
        assert_eq!(app.search_selected_index, Some(0));
        assert_eq!(app.scroll, 4);
        assert_eq!(app.focus, ReaderFocus::Reader);
    }

    #[test]
    fn search_navigation_wraps_between_matches() {
        let mut app = ReaderApp::new(Document::from_source(
            "Needle one.\n\nMiddle.\n\nNeedle two.",
        ));
        app.search_query = "needle".to_owned();
        app.submit_search();

        app.select_next_search_match();
        assert_eq!(app.search_selected_index, Some(1));
        assert_eq!(app.scroll, 4);

        app.select_next_search_match();
        assert_eq!(app.search_selected_index, Some(0));
        assert_eq!(app.scroll, 0);

        app.select_previous_search_match();
        assert_eq!(app.search_selected_index, Some(1));
    }

    #[test]
    fn presentation_mode_enters_and_renders_first_slide() {
        let mut app = ReaderApp::new(Document::from_source("# Intro\n\nWelcome\n\n---\n\n# Next"));

        app.start_presentation();

        assert_eq!(app.presentation_mode, PresentationMode::Presenting);
        assert_eq!(app.presentation_slide_index, 0);
        assert_eq!(
            app.presentation_deck.as_ref().map(|deck| deck.len()),
            Some(2)
        );
        assert!(app.presentation_lines.iter().any(|line| line == "# Intro"));
        assert!(
            app.header_status()
                .as_deref()
                .is_some_and(|status| status.contains("Slide 1/2 - Intro"))
        );
    }

    #[test]
    fn presentation_mode_navigates_and_clamps_slides() {
        let mut app = ReaderApp::new(Document::from_source(
            "# One\n\n---\n\n# Two\n\n---\n\n# Three",
        ));

        app.start_presentation();
        app.handle_presentation_key(KeyCode::Right);
        assert_eq!(app.presentation_slide_index, 1);
        assert!(app.presentation_lines.iter().any(|line| line == "# Two"));

        app.handle_presentation_key(KeyCode::Char('n'));
        app.handle_presentation_key(KeyCode::Right);
        assert_eq!(app.presentation_slide_index, 2);
        assert!(app.presentation_lines.iter().any(|line| line == "# Three"));

        app.handle_presentation_key(KeyCode::Left);
        assert_eq!(app.presentation_slide_index, 1);

        app.handle_presentation_key(KeyCode::Char('b'));
        app.handle_presentation_key(KeyCode::Left);
        assert_eq!(app.presentation_slide_index, 0);
    }

    #[test]
    fn presentation_mode_space_advances_slide() {
        let mut app = ReaderApp::new(Document::from_source("# One\n\n---\n\n# Two"));

        app.start_presentation();
        app.handle_presentation_key(KeyCode::Char(' '));

        assert_eq!(app.presentation_slide_index, 1);
    }

    #[test]
    fn presentation_mode_home_end_jump_between_bounds() {
        let mut app = ReaderApp::new(Document::from_source(
            "# One\n\n---\n\n# Two\n\n---\n\n# Three",
        ));

        app.start_presentation();
        app.handle_presentation_key(KeyCode::End);

        assert_eq!(app.presentation_slide_index, 2);
        assert!(app.presentation_lines.iter().any(|line| line == "# Three"));
        assert_eq!(app.presentation_pane_title(), "Presentation 3 / 3");

        app.handle_presentation_key(KeyCode::Home);

        assert_eq!(app.presentation_slide_index, 0);
        assert!(app.presentation_lines.iter().any(|line| line == "# One"));
        assert_eq!(app.presentation_pane_title(), "Presentation 1 / 3");
    }

    #[test]
    fn presentation_mode_escape_and_q_exit_to_reader() {
        let mut app = ReaderApp::new(Document::from_source("# Intro\n\n---\n\n# Next"));

        app.start_presentation();
        app.handle_presentation_key(KeyCode::Esc);

        assert_eq!(app.presentation_mode, PresentationMode::Inactive);
        assert!(app.presentation_deck.is_none());
        assert!(app.presentation_lines.is_empty());
        assert_eq!(app.scroll, 0);

        app.start_presentation();
        app.handle_presentation_key(KeyCode::Char('q'));

        assert_eq!(app.presentation_mode, PresentationMode::Inactive);
    }

    #[test]
    fn document_text_highlights_selected_search_line() {
        let lines = vec!["Needle one.".to_owned(), "Plain.".to_owned()];
        let rendered = document_text(
            &lines,
            SearchHighlights {
                query: "needle",
                matches: &[SearchMatch {
                    line_index: 0,
                    column: 0,
                    line: "Needle one.".to_owned(),
                }],
                selected_index: Some(0),
            },
        );
        let highlighted = rendered[0]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "Needle")
            .expect("highlighted match span");

        assert_eq!(highlighted.style, theme::search_selected_emphasis());
        assert!(highlighted.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn document_text_highlights_unselected_search_lines() {
        let lines = vec!["Needle one.".to_owned(), "Needle two.".to_owned()];
        let matches = vec![
            SearchMatch {
                line_index: 0,
                column: 0,
                line: "Needle one.".to_owned(),
            },
            SearchMatch {
                line_index: 1,
                column: 0,
                line: "Needle two.".to_owned(),
            },
        ];
        let rendered = document_text(
            &lines,
            SearchHighlights {
                query: "needle",
                matches: &matches,
                selected_index: Some(0),
            },
        );
        let highlighted = rendered[1]
            .spans
            .iter()
            .find(|span| span.content.as_ref() == "Needle")
            .expect("highlighted match span");

        assert_eq!(highlighted.style, theme::search_matched_emphasis());
        assert!(highlighted.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn search_selection_clamps_after_reload() {
        assert_eq!(clamp_search_selection(Some(3), 2), Some(1));
        assert_eq!(clamp_search_selection(Some(0), 0), None);
        assert_eq!(clamp_search_selection(None, 2), Some(0));
    }

    #[test]
    fn reader_can_open_near_workspace_search_line() {
        let app = ReaderApp::new_at_source_line(
            Document::from_source("# Title\n\nOne.\n\nNeedle.\n\nTail."),
            5,
        );

        assert_eq!(app.scroll, 4);
        assert_eq!(
            app.status.as_deref(),
            Some("Opened search result at line 5")
        );
    }

    #[test]
    fn workspace_search_selection_is_bounded() {
        let mut app = WorkspaceSearchApp::new(
            "needle".to_owned(),
            ".".into(),
            vec![
                WorkspaceSearchMatch {
                    path: "one.md".into(),
                    line_number: 1,
                    column: 1,
                    line: "Needle one".to_owned(),
                },
                WorkspaceSearchMatch {
                    path: "two.md".into(),
                    line_number: 2,
                    column: 3,
                    line: "Needle two".to_owned(),
                },
            ],
            HistoryStore::new("missing-history.toml"),
        );

        app.select_previous();
        assert_eq!(app.list_state.selected(), Some(0));

        app.select_next();
        app.select_next();
        assert_eq!(app.list_state.selected(), Some(1));
    }

    #[test]
    fn bookmark_selection_is_bounded() {
        assert_eq!(clamp_bookmark_selection(Some(3), 2), Some(1));
        assert_eq!(clamp_bookmark_selection(Some(0), 0), None);
        assert_eq!(clamp_bookmark_selection(None, 2), Some(0));
    }

    #[test]
    fn bookmark_picker_opens_selected_bookmark_near_line() {
        let stem = temp_stem("bookmark-picker-open");
        let document_path = std::env::temp_dir().join(format!("{stem}.md"));
        let bookmark_path = std::env::temp_dir().join(format!("{stem}-bookmarks.toml"));
        let history_path = std::env::temp_dir().join(format!("{stem}-history.toml"));
        fs::write(&document_path, "# Pick Me\n\nOne.\n\nTwo.\n\nThree.").expect("write document");
        let mut bookmarks = paperview_core::Bookmarks::new();
        bookmarks.add(paperview_core::Bookmark::new(&document_path, "Pick Me").with_source_line(5));
        let bookmark_store = paperview_core::BookmarkStore::new(&bookmark_path);
        bookmark_store.save(&bookmarks).expect("save bookmarks");
        let history_store = paperview_core::HistoryStore::new(&history_path);
        let mut app = BookmarkApp::new(bookmark_store, history_store);

        let (document, line_number) = app.open_selected().expect("open selected bookmark");

        assert_eq!(document.path(), Some(&document_path));
        assert_eq!(line_number, 5);
        let reader = ReaderApp::new_at_source_line_with_status(
            document,
            line_number,
            format!("Opened bookmark at line {line_number}"),
        );
        assert_eq!(reader.scroll, 4);
        assert_eq!(reader.status.as_deref(), Some("Opened bookmark at line 5"));

        fs::remove_file(document_path).expect("remove document");
        fs::remove_file(bookmark_path).expect("remove bookmarks");
        fs::remove_file(history_path).expect("remove history");
    }

    #[test]
    fn bookmark_picker_prunes_missing_entries() {
        let stem = temp_stem("bookmark-picker-prune");
        let document_path = std::env::temp_dir().join(format!("{stem}.md"));
        let missing_path = std::env::temp_dir().join(format!("{stem}-missing.md"));
        let bookmark_path = std::env::temp_dir().join(format!("{stem}-bookmarks.toml"));
        let history_path = std::env::temp_dir().join(format!("{stem}-history.toml"));
        fs::write(&document_path, "# Present").expect("write document");
        let mut bookmarks = paperview_core::Bookmarks::new();
        bookmarks.add(paperview_core::Bookmark::new(&document_path, "Present"));
        bookmarks.add(paperview_core::Bookmark::new(&missing_path, "Missing"));
        let bookmark_store = paperview_core::BookmarkStore::new(&bookmark_path);
        bookmark_store.save(&bookmarks).expect("save bookmarks");
        let history_store = paperview_core::HistoryStore::new(&history_path);

        let app = BookmarkApp::new(bookmark_store, history_store);

        assert_eq!(app.bookmarks.entries().len(), 1);
        assert_eq!(app.list_state.selected(), Some(0));

        fs::remove_file(document_path).expect("remove document");
        fs::remove_file(bookmark_path).expect("remove bookmarks");
    }

    #[test]
    fn dashboard_selection_is_bounded() {
        let mut app = DashboardApp::new(HistoryStore::new("missing-history.toml"));
        app.history.record(FileEntry::new("one.md", "One"));
        app.history.record(FileEntry::new("two.md", "Two"));
        app.list_state.select(Some(0));

        app.select_previous();
        assert_eq!(app.list_state.selected(), Some(0));

        app.select_next();
        app.select_next();
        assert_eq!(app.list_state.selected(), Some(1));
    }

    #[test]
    fn dashboard_prunes_missing_history_entries_on_load() {
        let existing = temp_doc("tui-history-existing.md", "# Existing");
        let missing = existing.with_file_name("tui-history-missing.md");
        let history_path = existing.with_file_name("tui-history.toml");
        let store = HistoryStore::new(&history_path);
        let mut history = paperview_core::History::new();
        history.record(FileEntry::new(&missing, "Missing"));
        history.record(FileEntry::new(&existing, "Existing"));
        store.save(&history).expect("save history");

        let app = DashboardApp::new(store.clone());

        assert_eq!(app.history.entries().len(), 1);
        assert_eq!(app.history.entries()[0].path(), existing.as_path());
        assert_eq!(
            store.load().expect("load pruned history").entries().len(),
            1
        );

        fs::remove_file(existing).expect("remove existing history file");
        fs::remove_file(history_path).expect("remove history file");
    }

    #[test]
    fn reload_path_updates_document_and_clamps_scroll() {
        let path = temp_doc("tui-live-reload.md", "# Before\n\nLine one.\n\nLine two.");
        let document = Document::open(&path).expect("open initial document");
        let mut app = ReaderApp::new(document);
        app.scroll_to_bottom();

        fs::write(&path, "# After\n\nShort.").expect("rewrite test document");
        app.reload_path(path.clone());

        assert_eq!(app.active_document().title(), "After");
        assert_eq!(app.scroll, app.max_scroll());
        assert!(
            app.status
                .as_deref()
                .is_some_and(|status| status.starts_with("Reloaded "))
        );

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn reload_path_updates_split_document_without_activating_it() {
        let first = temp_doc("tui-split-reload-first.md", "# First\n\nOne.");
        let second = temp_doc("tui-split-reload-second.md", "# Before\n\nTwo.");
        let first_document = Document::open(&first).expect("open first document");
        let second_document = Document::open(&second).expect("open second document");
        let mut app = ReaderApp::new_documents(vec![first_document, second_document]);
        app.toggle_split();

        fs::write(&second, "# After\n\nUpdated.").expect("rewrite split document");
        app.reload_path(second.clone());

        assert_eq!(app.active_document().title(), "First");
        assert_eq!(app.split_document().map(Document::title), Some("After"));
        assert!(
            app.status
                .as_deref()
                .is_some_and(|status| status.starts_with("Reloaded side "))
        );

        fs::remove_file(first).expect("remove first document");
        fs::remove_file(second).expect("remove second document");
    }

    #[test]
    fn edit_mode_appends_and_saves_source() {
        let path = temp_doc("tui-edit-save.md", "# Draft\n\nBody");
        let mut app = ReaderApp::new(Document::open(&path).expect("open document"));

        app.start_editing();
        assert_eq!(app.edit_mode, EditMode::Editing);
        assert!(
            app.edit_session
                .as_ref()
                .is_some_and(|session| !session.is_dirty())
        );

        app.handle_edit_key(key(KeyCode::Enter));
        for character in "Added".chars() {
            app.handle_edit_key(key(KeyCode::Char(character)));
        }

        assert!(
            app.edit_session
                .as_ref()
                .is_some_and(|session| session.is_dirty())
        );

        app.handle_edit_key(ctrl_key('s'));

        assert_eq!(
            fs::read_to_string(&path).expect("read saved source"),
            "# Draft\n\nBody\nAdded"
        );
        assert_eq!(app.active_document().source(), "# Draft\n\nBody\nAdded");
        assert_eq!(app.edit_mode, EditMode::Editing);
        assert!(
            app.edit_session
                .as_ref()
                .is_some_and(|session| !session.is_dirty())
        );

        fs::remove_file(path).expect("remove document");
    }

    #[test]
    fn edit_mode_escape_closes_without_saving() {
        let path = temp_doc("tui-edit-cancel.md", "# Draft\n\nBody");
        let mut app = ReaderApp::new(Document::open(&path).expect("open document"));

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Char('x')));
        app.handle_edit_key(key(KeyCode::Esc));

        assert_eq!(app.edit_mode, EditMode::Editing);
        assert!(app.edit_discard_pending);
        assert!(
            app.status
                .as_deref()
                .is_some_and(|status| status.contains("Unsaved edits"))
        );

        app.handle_edit_key(key(KeyCode::Esc));

        assert_eq!(app.edit_mode, EditMode::Inactive);
        assert!(app.edit_session.is_none());
        assert_eq!(
            fs::read_to_string(&path).expect("read source"),
            "# Draft\n\nBody"
        );
        assert_eq!(app.active_document().source(), "# Draft\n\nBody");

        fs::remove_file(path).expect("remove document");
    }

    #[test]
    fn edit_mode_clean_escape_closes_immediately() {
        let mut app = ReaderApp::new(Document::from_source("# Draft\n\nBody"));

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Esc));

        assert_eq!(app.edit_mode, EditMode::Inactive);
        assert!(!app.edit_discard_pending);
    }

    #[test]
    fn edit_mode_toggles_preview_visibility() {
        let mut app = ReaderApp::new(Document::from_source("# Draft\n\nBody"));

        app.start_editing();
        assert!(app.edit_preview_visible);

        app.handle_edit_key(ctrl_key('p'));

        assert!(!app.edit_preview_visible);
        assert!(
            app.header_status()
                .as_deref()
                .is_some_and(|status| status.contains("preview off"))
        );

        app.handle_edit_key(ctrl_key('p'));

        assert!(app.edit_preview_visible);
    }

    #[test]
    fn edit_mode_preview_visibility_resets_on_new_session() {
        let mut app = ReaderApp::new(Document::from_source("# Draft\n\nBody"));

        app.start_editing();
        app.handle_edit_key(ctrl_key('p'));
        assert!(!app.edit_preview_visible);

        app.handle_edit_key(key(KeyCode::Esc));
        app.start_editing();

        assert!(app.edit_preview_visible);
    }

    #[test]
    fn edit_mode_scrolls_preview_independently() {
        let mut app = ReaderApp::new(Document::from_source(&numbered_lines(40)));

        app.start_editing();
        app.handle_edit_key(ctrl_event(KeyCode::Down));

        assert_eq!(app.edit_preview_scroll, 1);
        assert_ne!(app.edit_preview_scroll, app.edit_scroll);

        app.handle_edit_key(ctrl_event(KeyCode::PageDown));
        assert!(app.edit_preview_scroll > 1);

        app.handle_edit_key(ctrl_event(KeyCode::PageUp));
        app.handle_edit_key(ctrl_event(KeyCode::Up));

        assert!(
            app.edit_preview_scroll
                <= edit_preview_max_scroll(&app.edit_preview_lines, EDIT_VIEWPORT_LINES)
        );
    }

    #[test]
    fn edit_mode_preview_scroll_resets_on_new_session() {
        let mut app = ReaderApp::new(Document::from_source(&numbered_lines(40)));

        app.start_editing();
        app.handle_edit_key(ctrl_event(KeyCode::PageDown));
        assert!(app.edit_preview_scroll > 0);

        app.handle_edit_key(key(KeyCode::Esc));
        app.start_editing();

        assert_eq!(app.edit_preview_scroll, 0);
    }

    #[test]
    fn edit_mode_preview_scroll_clamps_when_preview_shrinks() {
        let mut app = ReaderApp::new(Document::from_source(&numbered_lines(40)));

        app.start_editing();
        app.handle_edit_key(ctrl_event(KeyCode::PageDown));
        app.handle_edit_key(ctrl_event(KeyCode::PageDown));
        assert!(app.edit_preview_scroll > 0);

        app.edit_buffer = "short".to_owned();
        app.edit_cursor = app.edit_buffer.len();
        app.refresh_edit_session();

        assert_eq!(app.edit_preview_scroll, 0);
    }

    #[test]
    fn edit_mode_hidden_preview_keeps_scroll_state() {
        let mut app = ReaderApp::new(Document::from_source(&numbered_lines(40)));

        app.start_editing();
        app.handle_edit_key(ctrl_key('p'));
        app.handle_edit_key(ctrl_event(KeyCode::Down));

        assert!(!app.edit_preview_visible);
        assert_eq!(app.edit_preview_scroll, 1);
    }

    #[test]
    fn edit_mode_save_clears_discard_warning() {
        let path = temp_doc("tui-edit-save-clears-discard.md", "# Draft\n\nBody");
        let mut app = ReaderApp::new(Document::open(&path).expect("open document"));

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Char('x')));
        app.handle_edit_key(key(KeyCode::Esc));
        assert!(app.edit_discard_pending);

        app.handle_edit_key(ctrl_key('s'));

        assert!(!app.edit_discard_pending);
        assert_eq!(app.edit_mode, EditMode::Editing);

        fs::remove_file(path).expect("remove document");
    }

    #[test]
    fn dirty_edit_blocks_tab_switch_until_confirmed() {
        let first = Document::from_source("# First");
        let second = Document::from_source("# Second");
        let mut app = ReaderApp::new_documents(vec![first, second]);

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Char('x')));
        app.select_next_tab();

        assert_eq!(app.documents.active_index(), Some(0));
        assert_eq!(app.edit_mode, EditMode::Editing);
        assert!(app.edit_discard_pending);

        app.select_next_tab();

        assert_eq!(app.documents.active_index(), Some(1));
        assert_eq!(app.edit_mode, EditMode::Inactive);
    }

    #[test]
    fn dirty_edit_blocks_close_until_confirmed() {
        let first = Document::from_source("# First");
        let second = Document::from_source("# Second");
        let mut app = ReaderApp::new_documents(vec![first, second]);

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Char('x')));

        assert!(!app.close_active_tab());
        assert_eq!(app.documents.len(), 2);
        assert_eq!(app.edit_mode, EditMode::Editing);
        assert!(app.edit_discard_pending);

        assert!(!app.close_active_tab());
        assert_eq!(app.documents.len(), 1);
        assert_eq!(app.edit_mode, EditMode::Inactive);
    }

    #[test]
    fn edit_mode_inserts_and_deletes_at_cursor() {
        let mut app = ReaderApp::new(Document::from_source("abc"));

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Left));
        app.handle_edit_key(key(KeyCode::Char('X')));
        assert_eq!(app.edit_buffer, "abXc");

        app.handle_edit_key(key(KeyCode::Left));
        app.handle_edit_key(key(KeyCode::Backspace));
        assert_eq!(app.edit_buffer, "aXc");

        app.handle_edit_key(key(KeyCode::Delete));
        assert_eq!(app.edit_buffer, "ac");
    }

    #[test]
    fn edit_mode_moves_cursor_by_line() {
        let mut app = ReaderApp::new(Document::from_source("one\ntwo\nthree"));

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Home));
        app.handle_edit_key(key(KeyCode::Up));
        app.handle_edit_key(key(KeyCode::Char('X')));
        assert_eq!(app.edit_buffer, "one\nXtwo\nthree");

        app.handle_edit_key(key(KeyCode::End));
        app.handle_edit_key(key(KeyCode::Up));
        app.handle_edit_key(key(KeyCode::Char('Y')));
        assert_eq!(app.edit_buffer, "oneY\nXtwo\nthree");
    }

    #[test]
    fn edit_mode_cursor_handles_multibyte_text() {
        let mut app = ReaderApp::new(Document::from_source("aé文"));

        app.start_editing();
        app.handle_edit_key(key(KeyCode::Left));
        app.handle_edit_key(key(KeyCode::Left));
        app.handle_edit_key(key(KeyCode::Char('X')));

        assert_eq!(app.edit_buffer, "aXé文");
    }

    #[test]
    fn edit_mode_preview_updates_before_save() {
        let path = temp_doc("tui-edit-preview.md", "# Draft\n\nBody");
        let mut app = ReaderApp::new(Document::open(&path).expect("open document"));

        app.start_editing();
        assert!(app.edit_preview_lines.iter().any(|line| line == "# Draft"));

        for character in "\n\n## Preview".chars() {
            app.handle_edit_key(key(if character == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(character)
            }));
        }

        assert!(
            app.edit_preview_lines
                .iter()
                .any(|line| line.contains("Preview"))
        );
        assert_eq!(
            fs::read_to_string(&path).expect("read unsaved source"),
            "# Draft\n\nBody"
        );

        fs::remove_file(path).expect("remove document");
    }

    #[test]
    fn edit_mode_scrolls_to_keep_cursor_visible() {
        let source = numbered_lines(30);
        let mut app = ReaderApp::new(Document::from_source(&source));

        app.start_editing();
        assert_eq!(app.scroll, 0);
        assert!(app.edit_scroll > 0);
        assert!(edit_cursor_line_is_visible(&app));

        app.handle_edit_key(key(KeyCode::Home));
        app.handle_edit_key(key(KeyCode::Char('X')));

        assert!(edit_cursor_line_is_visible(&app));
    }

    #[test]
    fn edit_mode_page_keys_move_cursor_and_scroll() {
        let source = numbered_lines(40);
        let mut app = ReaderApp::new(Document::from_source(&source));

        app.start_editing();
        let initial_cursor = app.edit_cursor;

        app.handle_edit_key(key(KeyCode::PageUp));

        assert!(app.edit_cursor < initial_cursor);
        assert!(edit_cursor_line_is_visible(&app));

        app.handle_edit_key(key(KeyCode::PageDown));

        assert!(app.edit_cursor > 0);
        assert!(edit_cursor_line_is_visible(&app));
    }

    #[test]
    fn edit_mode_close_clears_preview_lines() {
        let mut app = ReaderApp::new(Document::from_source("# Draft\n\nBody"));

        app.start_editing();
        assert!(!app.edit_preview_lines.is_empty());

        app.handle_edit_key(key(KeyCode::Esc));

        assert!(app.edit_preview_lines.is_empty());
    }

    fn temp_doc(name: &str, source: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("paperview-tui-{nanos}-{name}"));

        fs::write(&path, source).expect("write test document");

        path
    }

    fn temp_stem(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        format!("paperview-tui-{nanos}-{name}")
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(character: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(character), KeyModifiers::CONTROL)
    }

    fn ctrl_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn numbered_lines(count: usize) -> String {
        (0..count)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn edit_cursor_line_is_visible(app: &ReaderApp) -> bool {
        let (cursor_line, _) = cursor_line_column(&app.edit_buffer, app.edit_cursor);
        let scroll = usize::from(app.edit_scroll);
        (scroll..scroll + EDIT_VIEWPORT_LINES).contains(&cursor_line)
    }
}
