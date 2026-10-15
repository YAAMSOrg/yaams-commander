//! Lucide icon font helpers.

use iced::widget::{Text, text};

pub use lucide_icons::Icon;

pub const FONT: iced::Font = iced::Font::with_name("lucide");
pub const FONT_BYTES: &[u8] = lucide_icons::LUCIDE_FONT_BYTES;

/// An icon glyph at the given size.
pub fn icon<'a>(icon: Icon, size: impl Into<iced::Pixels>) -> Text<'a> {
    text(char::from(icon).to_string())
        .font(FONT)
        .size(size)
        .line_height(1.0)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
}
