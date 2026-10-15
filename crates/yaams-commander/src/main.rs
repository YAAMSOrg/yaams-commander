// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod assets;
mod config;
mod format;
mod icons;
mod screens;
mod theme;
mod validation;
mod widgets;

#[cfg(test)]
mod ui_tests;

use iced::{Size, window};

fn main() -> iced::Result {
    let mut application = iced::application(app::App::new, app::App::update, app::App::view)
        .title(app::App::title)
        .subscription(app::App::subscription)
        .theme(app::App::theme)
        .default_font(theme::FONT_BODY)
        .antialiasing(true)
        .window(window::Settings {
            size: Size::new(1440.0, 900.0),
            min_size: Some(Size::new(1180.0, 720.0)),
            icon: window::icon::from_file_data(assets::APP_ICON_PNG, None).ok(),
            ..window::Settings::default()
        })
        .centered();
    for font in assets::FONTS {
        application = application.font(font);
    }
    application.run()
}
