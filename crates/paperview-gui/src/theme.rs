use iced::{Background, Border, Color, border, widget::container};

pub const SHELL_BACKGROUND: Color = Color::from_rgb(0.067, 0.075, 0.094);
pub const SHELL_SURFACE: Color = Color::from_rgb(0.086, 0.102, 0.133);
pub const SHELL_TEXT: Color = Color::WHITE;
pub const SHELL_TEXT_MUTED: Color = Color::from_rgb(0.545, 0.58, 0.62);
pub const SHELL_ACCENT: Color = Color::from_rgb(0.345, 0.651, 1.0);
pub const READER_BACKGROUND: Color = Color::from_rgb(0.992, 0.973, 0.937);
pub const READER_TEXT: Color = Color::from_rgb(0.122, 0.137, 0.157);
pub const READER_TEXT_MUTED: Color = Color::from_rgb(0.325, 0.345, 0.373);
const READER_BORDER: Color = Color::from_rgb(0.816, 0.843, 0.871);
const CODE_BACKGROUND: Color = Color::from_rgb(0.965, 0.973, 0.98);

pub fn application_style() -> iced::theme::Style {
    iced::theme::Style {
        background_color: SHELL_BACKGROUND,
        text_color: SHELL_TEXT,
    }
}

pub fn shell_container() -> container::Style {
    container::Style::default()
        .background(SHELL_BACKGROUND)
        .color(SHELL_TEXT)
}

pub fn header_container() -> container::Style {
    container::Style::default()
        .background(SHELL_SURFACE)
        .color(SHELL_TEXT)
}

pub fn reader_backdrop() -> container::Style {
    container::Style::default()
        .background(SHELL_BACKGROUND)
        .color(READER_TEXT)
}

pub fn paper_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(READER_BACKGROUND)),
        text_color: Some(READER_TEXT),
        border: Border {
            color: READER_BORDER,
            width: 1.0,
            radius: border::radius(6),
        },
        ..container::Style::default()
    }
}

pub fn quote_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(CODE_BACKGROUND)),
        text_color: Some(READER_TEXT_MUTED),
        border: Border {
            color: SHELL_ACCENT,
            width: 1.0,
            radius: border::radius(4),
        },
        ..container::Style::default()
    }
}

pub fn code_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(CODE_BACKGROUND)),
        text_color: Some(READER_TEXT),
        border: Border {
            color: READER_BORDER,
            width: 1.0,
            radius: border::radius(6),
        },
        ..container::Style::default()
    }
}
