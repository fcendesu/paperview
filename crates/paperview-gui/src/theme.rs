use iced::{
    Background, Border, Color, Shadow, Vector, border,
    widget::{button, container, text_input},
};

pub const SHELL_BACKGROUND: Color = Color::from_rgb(0.067, 0.075, 0.094);
pub const SHELL_SURFACE: Color = Color::from_rgb(0.086, 0.102, 0.133);
pub const SHELL_ACTIVE_SURFACE: Color = Color::from_rgb(0.106, 0.122, 0.153);
pub const SHELL_TEXT: Color = Color::WHITE;
pub const SHELL_TEXT_MUTED: Color = Color::from_rgb(0.545, 0.58, 0.62);
pub const SHELL_ACCENT: Color = Color::from_rgb(0.345, 0.651, 1.0);
pub const READER_BACKGROUND: Color = Color::from_rgb(0.992, 0.973, 0.937);
pub const READER_TEXT: Color = Color::from_rgb(0.122, 0.137, 0.157);
pub const READER_TEXT_MUTED: Color = Color::from_rgb(0.325, 0.345, 0.373);
pub const SEARCH_HIGHLIGHT_BACKGROUND: Color = Color::from_rgb(1.0, 0.847, 0.298);
pub const SEARCH_HIGHLIGHT_TEXT: Color = Color::from_rgb(0.067, 0.075, 0.094);
const READER_BORDER: Color = Color::from_rgb(0.816, 0.843, 0.871);
pub const CODE_BACKGROUND: Color = Color::from_rgb(0.965, 0.973, 0.98);
const PAPER_SHADOW: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.22,
};

pub fn application_style() -> iced::theme::Style {
    iced::theme::Style {
        background_color: SHELL_BACKGROUND,
        text_color: SHELL_TEXT,
    }
}

pub fn shell_container(is_drag_hovered: bool) -> container::Style {
    let border = if is_drag_hovered {
        Border {
            color: SHELL_ACCENT,
            width: 2.0,
            radius: border::radius(0),
        }
    } else {
        Border::default()
    };

    container::Style {
        background: Some(Background::Color(SHELL_BACKGROUND)),
        text_color: Some(SHELL_TEXT),
        border,
        ..container::Style::default()
    }
}

pub fn header_container() -> container::Style {
    container::Style::default()
        .background(SHELL_SURFACE)
        .color(SHELL_TEXT)
}

pub fn tab_bar_container() -> container::Style {
    container::Style::default()
        .background(SHELL_BACKGROUND)
        .color(SHELL_TEXT_MUTED)
}

pub fn navigation_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(SHELL_SURFACE)),
        text_color: Some(SHELL_TEXT),
        border: Border {
            color: SHELL_BACKGROUND,
            width: 1.0,
            radius: border::radius(0),
        },
        ..container::Style::default()
    }
}

pub fn history_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(SHELL_SURFACE)),
        text_color: Some(SHELL_TEXT),
        border: Border {
            color: SHELL_BACKGROUND,
            width: 1.0,
            radius: border::radius(0),
        },
        ..container::Style::default()
    }
}

pub fn history_item_button(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => SHELL_ACCENT,
        button::Status::Active | button::Status::Disabled => SHELL_ACTIVE_SURFACE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: SHELL_TEXT,
        border: Border {
            color: background,
            width: 1.0,
            radius: border::radius(6),
        },
        ..button::Style::default()
    }
}

pub fn toc_item_button(is_active: bool, status: button::Status) -> button::Style {
    let background = if is_active {
        SHELL_ACTIVE_SURFACE
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => SHELL_ACTIVE_SURFACE,
            button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        }
    };
    let text_color = if is_active {
        SHELL_ACCENT
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        SHELL_TEXT
    } else {
        SHELL_TEXT_MUTED
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: background,
            width: 1.0,
            radius: border::radius(4),
        },
        ..button::Style::default()
    }
}

pub fn header_action_button(is_active: bool, status: button::Status) -> button::Style {
    let background = if is_active {
        SHELL_ACCENT
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => SHELL_ACTIVE_SURFACE,
            button::Status::Active => SHELL_BACKGROUND,
            button::Status::Disabled => SHELL_SURFACE,
        }
    };
    let text_color = if matches!(status, button::Status::Disabled) {
        SHELL_TEXT_MUTED
    } else {
        SHELL_TEXT
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: if is_active { SHELL_ACCENT } else { background },
            width: 1.0,
            radius: border::radius(6),
        },
        ..button::Style::default()
    }
}

pub fn search_input(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let is_focused = matches!(status, text_input::Status::Focused { .. });
    let border_color = if is_focused {
        SHELL_ACCENT
    } else {
        SHELL_ACTIVE_SURFACE
    };

    text_input::Style {
        background: Background::Color(SHELL_BACKGROUND),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(6),
        },
        icon: SHELL_TEXT_MUTED,
        placeholder: SHELL_TEXT_MUTED,
        value: SHELL_TEXT,
        selection: SHELL_ACCENT,
    }
}

pub fn tab_button(is_active: bool, status: button::Status) -> button::Style {
    let background = if is_active {
        READER_BACKGROUND
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => SHELL_ACTIVE_SURFACE,
            button::Status::Active | button::Status::Disabled => SHELL_BACKGROUND,
        }
    };
    let text_color = if is_active {
        READER_TEXT
    } else {
        SHELL_TEXT_MUTED
    };
    let border_color = if is_active { SHELL_ACCENT } else { background };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: border::radius(6),
        },
        ..button::Style::default()
    }
}

pub fn split_tab_button(is_selected: bool, status: button::Status) -> button::Style {
    let background = if is_selected {
        SHELL_ACCENT
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => SHELL_ACTIVE_SURFACE,
            button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        }
    };
    let text_color =
        if is_selected || matches!(status, button::Status::Hovered | button::Status::Pressed) {
            SHELL_TEXT
        } else {
            SHELL_TEXT_MUTED
        };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: if is_selected {
                SHELL_ACCENT
            } else {
                background
            },
            width: 1.0,
            radius: border::radius(4),
        },
        ..button::Style::default()
    }
}

pub fn tab_close_button(is_active: bool, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => SHELL_ACCENT,
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
    };
    let text_color = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        SHELL_TEXT
    } else if is_active {
        READER_TEXT_MUTED
    } else {
        SHELL_TEXT_MUTED
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: background,
            width: 1.0,
            radius: border::radius(4),
        },
        ..button::Style::default()
    }
}

pub fn inactive_tab_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(SHELL_ACTIVE_SURFACE)),
        text_color: Some(SHELL_TEXT_MUTED),
        border: Border {
            color: SHELL_ACTIVE_SURFACE,
            width: 1.0,
            radius: border::radius(6),
        },
        ..container::Style::default()
    }
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
        shadow: Shadow {
            color: PAPER_SHADOW,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
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

pub fn math_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.98, 0.965, 0.925))),
        text_color: Some(READER_TEXT),
        border: Border {
            color: Color::from_rgb(0.706, 0.576, 0.306),
            width: 1.0,
            radius: border::radius(6),
        },
        ..container::Style::default()
    }
}

pub fn diagram_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.945, 0.976, 0.968))),
        text_color: Some(READER_TEXT),
        border: Border {
            color: Color::from_rgb(0.196, 0.639, 0.565),
            width: 1.0,
            radius: border::radius(6),
        },
        ..container::Style::default()
    }
}

pub fn image_container() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.953, 0.961, 0.984))),
        text_color: Some(READER_TEXT),
        border: Border {
            color: Color::from_rgb(0.455, 0.533, 0.761),
            width: 1.0,
            radius: border::radius(6),
        },
        ..container::Style::default()
    }
}

pub fn table_container() -> container::Style {
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

pub fn table_cell_container(is_header: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(if is_header {
            Color::from_rgb(0.922, 0.941, 0.961)
        } else {
            CODE_BACKGROUND
        })),
        text_color: Some(READER_TEXT),
        border: Border {
            color: READER_BORDER,
            width: 0.5,
            radius: border::radius(0),
        },
        ..container::Style::default()
    }
}
