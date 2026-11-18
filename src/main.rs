// Mine — Entry point: Personal Finance Manager (iced + Windows XP style)

mod app;
mod chart;
mod db;
mod export;
mod models;
mod theme;
mod util;

use iced::{Font, Size, window};

fn main() -> iced::Result {
    iced::application(app::App::new, app::App::update, app::App::view)
        .title("Finance Manager")
        .theme(|_: &app::App| theme::xp_theme())
        .default_font(Font::with_name("Tahoma"))
        .window(window::Settings {
            size: Size::new(1100.0, 720.0),
            min_size: Some(Size::new(900.0, 600.0)),
            ..Default::default()
        })
        .run()
}
