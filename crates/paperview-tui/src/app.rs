use std::{
    io,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use paperview_core::{
    Document, FileEntry, FileWatcher, History, HistoryStore, SearchMatch, WatchEvent,
    parser::TocItem, watch_file,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::render;

pub fn run(document: Document) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = ReaderApp::new(document).run(&mut terminal);
    ratatui::restore();
    result
}

pub fn run_dashboard() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = DashboardApp::new(HistoryStore::default()).run(&mut terminal);
    ratatui::restore();
    result
}

struct ReaderApp {
    document: Document,
    document_lines: Vec<String>,
    block_line_starts: Vec<render::BlockLineStart>,
    toc: Vec<TocItem>,
    toc_selected_index: Option<usize>,
    focus: ReaderFocus,
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
        let rendered = render::render_document_with_anchors(&document);
        let toc = document.parsed().toc();
        let (watcher, watch_receiver, status) = watch_document(&document);

        Self {
            document,
            document_lines: rendered.lines,
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
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(frame.area());
        let [reader, toc] =
            Layout::horizontal([Constraint::Min(50), Constraint::Length(32)]).areas(body);

        let title = format!(" PaperView - {} ", self.document.title());
        frame.render_widget(
            Paragraph::new(title)
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .block(Block::default().borders(Borders::BOTTOM)),
            header,
        );

        if let Some(status) = self.header_status() {
            let status_area = header.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 2,
            });
            frame.render_widget(
                Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
                status_area,
            );
        }

        frame.render_widget(
            Paragraph::new(Text::from(document_text(&self.document_lines)))
                .block(Block::default().title("Reader").borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray))
                .scroll((self.scroll, 0))
                .wrap(Wrap { trim: false }),
            reader,
        );

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
        self.search_matches = self.document.search(&self.search_query);
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

    fn toggle_focus(&mut self) {
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
        if self.document.path() != Some(&path) {
            return;
        }

        match Document::open(&path) {
            Ok(document) => {
                let rendered = render::render_document_with_anchors(&document);
                self.document_lines = rendered.lines;
                self.block_line_starts = rendered.block_line_starts;
                self.toc = document.parsed().toc();
                self.toc_selected_index =
                    clamp_toc_selection(self.toc_selected_index, self.toc.len());
                self.refresh_search_matches();
                if self.toc.is_empty() {
                    self.focus = ReaderFocus::Reader;
                }
                self.scroll = self.scroll.min(self.max_scroll());
                self.document = document;
                self.status = Some(format!("Reloaded {}", path.display()));
            }
            Err(error) => {
                self.status = Some(error.to_string());
            }
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
        let history = store.load().unwrap_or_else(|error| {
            eprintln!("{error}");
            History::new()
        });
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
                            self.history = self.store.load().unwrap_or_else(|error| {
                                eprintln!("{error}");
                                History::new()
                            });
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
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .block(Block::default().borders(Borders::BOTTOM)),
            header,
        );

        if self.history.is_empty() {
            frame.render_widget(
                Paragraph::new("No recent files yet.\n\nOpen a file with paperview-tui <file>.")
                    .block(Block::default().title("History").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Gray))
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
                .highlight_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_stateful_widget(list, body, &mut self.list_state);
        }

        let status = self
            .status
            .as_deref()
            .unwrap_or("Enter opens selected file - j/k move - q quits");
        frame.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            footer,
        );
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

fn history_item(entry: &FileEntry) -> ListItem<'static> {
    ListItem::new(vec![
        Line::from(Span::styled(
            entry.title().to_owned(),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            entry.path().display().to_string(),
            Style::default().fg(Color::DarkGray),
        )),
    ])
}

fn document_text(lines: &[String]) -> Vec<Line<'static>> {
    lines.iter().map(|line| document_line(line)).collect()
}

fn document_line(line: &str) -> Line<'static> {
    if line.starts_with('#') {
        Line::from(Span::styled(
            line.to_owned(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
    } else if line.starts_with("> ") {
        Line::from(Span::styled(
            line.to_owned(),
            Style::default().fg(Color::Blue),
        ))
    } else {
        Line::from(line.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crossterm::event::KeyCode;
    use paperview_core::{Document, FileEntry, HistoryStore};

    use super::{
        DashboardApp, ReaderApp, ReaderFocus, SearchMode, clamp_search_selection,
        clamp_toc_selection,
    };

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
    fn search_selection_clamps_after_reload() {
        assert_eq!(clamp_search_selection(Some(3), 2), Some(1));
        assert_eq!(clamp_search_selection(Some(0), 0), None);
        assert_eq!(clamp_search_selection(None, 2), Some(0));
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
    fn reload_path_updates_document_and_clamps_scroll() {
        let path = temp_doc("tui-live-reload.md", "# Before\n\nLine one.\n\nLine two.");
        let document = Document::open(&path).expect("open initial document");
        let mut app = ReaderApp::new(document);
        app.scroll_to_bottom();

        fs::write(&path, "# After\n\nShort.").expect("rewrite test document");
        app.reload_path(path.clone());

        assert_eq!(app.document.title(), "After");
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
