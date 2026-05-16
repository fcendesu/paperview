mod app;
mod history;
mod navigation;
mod reader;
mod theme;

use std::env;

fn main() -> iced::Result {
    let initial_state = app::PaperView::from_args(env::args_os().skip(1));

    iced::application(move || initial_state.clone(), app::update, app::view)
        .title(app::title)
        .theme(app::iced_theme)
        .style(app::style)
        .window_size(iced::Size::new(1120.0, 760.0))
        .centered()
        .antialiasing(true)
        .run()
}
