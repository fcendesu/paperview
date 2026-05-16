use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use paperview_core::{Document, FileEntry, History, HistoryStore};
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

#[derive(Debug)]
struct ReaderApp {
    document: Document,
    document_lines: Vec<String>,
    toc_lines: Vec<String>,
    scroll: u16,
}

impl ReaderApp {
    fn new(document: Document) -> Self {
        let document_lines = render::render_document_lines(&document);
        let toc_lines = render::render_toc_lines(&document.parsed().toc());

        Self {
            document,
            document_lines,
            toc_lines,
            scroll: 0,
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
                    KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                    KeyCode::Char('g') => self.scroll = 0,
                    KeyCode::Char('G') => self.scroll_to_bottom(),
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

        frame.render_widget(
            Paragraph::new(Text::from(document_text(&self.document_lines)))
                .block(Block::default().title("Reader").borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray))
                .scroll((self.scroll, 0))
                .wrap(Wrap { trim: false }),
            reader,
        );

        frame.render_widget(
            Paragraph::new(Text::from(toc_text(&self.toc_lines)))
                .block(Block::default().title("On this page").borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: true }),
            toc,
        );
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

    fn max_scroll(&self) -> u16 {
        self.document_lines.len().saturating_sub(1) as u16
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

fn toc_text(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .skip(2)
        .map(|line| Line::from(line.to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use paperview_core::{Document, FileEntry, HistoryStore};

    use super::{DashboardApp, ReaderApp};

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
}
