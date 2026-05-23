use ratatui::style::{Color, Modifier, Style};

const SHELL_BG: Color = Color::Rgb(17, 19, 24);
const READER_BG: Color = Color::Rgb(253, 248, 239);
const READER_TEXT: Color = Color::Rgb(31, 35, 40);
const MUTED_TEXT: Color = Color::Rgb(139, 148, 158);
const ACCENT: Color = Color::Rgb(88, 166, 255);
const SEARCH_BG: Color = Color::Rgb(255, 221, 87);

pub fn shell() -> Style {
    Style::default().fg(Color::White).bg(SHELL_BG)
}

pub fn shell_muted() -> Style {
    Style::default().fg(MUTED_TEXT).bg(SHELL_BG)
}

pub fn reader() -> Style {
    Style::default().fg(READER_TEXT).bg(READER_BG)
}

pub fn reader_heading() -> Style {
    reader().add_modifier(Modifier::BOLD)
}

pub fn reader_quote() -> Style {
    reader().fg(ACCENT)
}

pub fn tab_active() -> Style {
    reader().add_modifier(Modifier::BOLD)
}

pub fn tab_inactive() -> Style {
    shell_muted()
}

pub fn zen_badge() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(SHELL_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn status() -> Style {
    Style::default().fg(MUTED_TEXT)
}

pub fn list_title() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn list_meta() -> Style {
    Style::default().fg(MUTED_TEXT)
}

pub fn list_highlight() -> Style {
    Style::default()
        .fg(Color::White)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn toc_empty() -> Style {
    Style::default().fg(MUTED_TEXT)
}

pub fn toc_active() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn toc_selected() -> Style {
    list_highlight()
}

pub fn toc_inactive() -> Style {
    Style::default().fg(MUTED_TEXT)
}

pub fn search_selected() -> Style {
    Style::default().fg(READER_TEXT).bg(SEARCH_BG)
}

pub fn search_selected_emphasis() -> Style {
    search_selected().add_modifier(Modifier::BOLD)
}

pub fn search_matched() -> Style {
    Style::default().fg(Color::White).bg(Color::DarkGray)
}

pub fn search_matched_emphasis() -> Style {
    Style::default()
        .fg(Color::White)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_maps_reader_and_shell_to_distinct_surfaces() {
        assert_ne!(shell().bg, reader().bg);
        assert_ne!(shell_muted().fg, reader().fg);
    }

    #[test]
    fn theme_emphasizes_active_navigation_and_search() {
        assert!(tab_active().add_modifier.contains(Modifier::BOLD));
        assert!(toc_selected().add_modifier.contains(Modifier::BOLD));
        assert!(
            search_selected_emphasis()
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }
}
