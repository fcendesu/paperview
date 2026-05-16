use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use paperview_core::Document;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::render;

pub fn run(document: Document) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = TuiApp::new(document).run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Debug)]
struct TuiApp {
    document: Document,
    document_lines: Vec<String>,
    toc_lines: Vec<String>,
    scroll: u16,
}

impl TuiApp {
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
    use paperview_core::Document;

    use super::TuiApp;

    #[test]
    fn scrolling_is_saturating() {
        let mut app = TuiApp::new(Document::from_source("# Title\n\nBody"));

        app.scroll_up();
        assert_eq!(app.scroll, 0);

        app.scroll_to_bottom();
        let bottom = app.scroll;
        app.scroll_down();
        assert_eq!(app.scroll, bottom);
    }
}
