use std::{
    fs, io,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use paperview_core::{
    Config, ConfigStore, Document, FileEntry, FileWatcher, History, HistoryStore, OpenDocuments,
    SearchMatch, SplitResize, SplitViewState, WatchEvent, WorkspaceSearchMatch,
    parser::{Block as MarkdownBlock, TocItem},
    toggle_task_line_source, watch_file,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{render, theme};

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

struct ReaderApp {
    config: Config,
    config_store: ConfigStore,
    documents: OpenDocuments,
    document_lines: Vec<String>,
    split_view: SplitViewState,
    split_document_lines: Vec<String>,
    block_line_starts: Vec<render::BlockLineStart>,
    toc: Vec<TocItem>,
    toc_selected_index: Option<usize>,
    focus: ReaderFocus,
    is_zen: bool,
    scroll: u16,
    status: Option<String>,
    search_mode: SearchMode,
    search_query: String,
    search_matches: Vec<SearchMatch>,
    search_selected_index: Option<usize>,
    _watcher: Option<FileWatcher>,
    watch_receiver: Option<Receiver<WatchEvent>>,
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
            is_zen: config.zen_mode,
            config,
            config_store,
            documents: open_documents,
            document_lines: rendered.lines,
            split_document_lines: Vec::new(),
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
            _watcher: watcher,
            watch_receiver,
        }
    }

    fn new_at_source_line(document: Document, line_number: usize) -> Self {
        let mut app = Self::new(document);
        app.scroll = line_number.saturating_sub(1) as u16;
        app.scroll = app.scroll.min(app.max_scroll());
        app.status = Some(format!("Opened search result at line {line_number}"));
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

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('/') => self.start_search(),
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
                    KeyCode::Char('g') if self.focus == ReaderFocus::Reader => self.scroll = 0,
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
        let (main, toc) = if self.is_zen {
            (body, None)
        } else {
            let [main, toc] =
                Layout::horizontal([Constraint::Min(50), Constraint::Length(32)]).areas(body);
            (main, Some(toc))
        };
        let reader_areas = if self.split_view.is_enabled() && !self.is_zen {
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

        if let Some(split_index) = self.split_view.secondary_index() {
            if self.is_zen {
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
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll = self.max_scroll();
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
            if self.is_zen {
                Line::from(Span::styled(" Zen Mode ".to_owned(), theme::zen_badge()))
            } else {
                tab_line(&self.documents)
            },
            Line::from(Span::styled(
                self.header_status().unwrap_or_else(|| {
                    "[/] search  [Space] task  [z] zen  [\\] split  [</>] resize  [{/}] side  [[/]] tabs  [x] close  [Tab] toc  [q] quit"
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
        self.is_zen = !self.is_zen;
        if self.is_zen {
            self.focus = ReaderFocus::Reader;
        }
        self.status = if self.is_zen {
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

        self._watcher = watcher;
        self.watch_receiver = watch_receiver;
        if let Some(status) = watch_status {
            self.status = Some(status);
        }
        self.ensure_split_target();
    }

    fn toggle_focus(&mut self) {
        if self.is_zen {
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

    fn handle_watch_events(&mut self) {
        let Some(receiver) = self.watch_receiver.take() else {
            return;
        };

        while let Ok(event) = receiver.try_recv() {
            match event {
                WatchEvent::Changed(path) => self.reload_path(path),
            }
        }

        self.watch_receiver = Some(receiver);
    }

    fn reload_path(&mut self, path: std::path::PathBuf) {
        if self.active_document().path() != Some(&path) {
            return;
        }

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
    }

    fn save_config(&mut self) {
        self.config.zen_mode = self.is_zen;
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

#[derive(Debug)]
struct WorkspaceSearchApp {
    query: String,
    root: std::path::PathBuf,
    matches: Vec<WorkspaceSearchMatch>,
    list_state: ListState,
    store: HistoryStore,
    status: Option<String>,
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

    use crossterm::event::KeyCode;
    use paperview_core::{
        Config, ConfigStore, Document, FileEntry, HistoryStore, SearchMatch, ThemePreference,
        WorkspaceSearchMatch,
    };
    use ratatui::style::Modifier;

    use super::{
        DashboardApp, ReaderApp, ReaderFocus, SearchHighlights, SearchMode, WorkspaceSearchApp,
        clamp_search_selection, clamp_toc_selection, document_text, tab_line,
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
        assert!(app.is_zen);
        assert_eq!(app.focus, ReaderFocus::Reader);

        app.toggle_focus();
        assert_eq!(app.focus, ReaderFocus::Reader);

        app.toggle_zen();
        assert!(!app.is_zen);
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
            })
            .expect("save config");

        let app = ReaderApp::new_documents_with_config(
            vec![Document::from_source("# First").with_path("first.md")],
            store,
        );

        assert!(app.is_zen);
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
    fn split_toggle_targets_first_non_active_tab() {
        let mut app = ReaderApp::new_documents(vec![
            Document::from_source("# First").with_path("first.md"),
            Document::from_source("# Second\n\nSide").with_path("second.md"),
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

    fn temp_doc(name: &str, source: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("paperview-tui-{nanos}-{name}"));

        fs::write(&path, source).expect("write test document");

        path
    }
}
