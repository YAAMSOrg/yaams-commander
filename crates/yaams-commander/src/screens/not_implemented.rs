//! Placeholder for areas Commander does not cover yet.

use iced::widget::row;
use iced::{Element, Length};

use crate::icons::Icon;
use crate::theme;
use crate::widgets::{action, empty_state};

pub fn view<'a, M: Clone + 'a>(title: &'a str, glyph: Icon, summary: &'a str, open_web: M) -> Element<'a, M> {
    let actions =
        row![action(Some(Icon::ExternalLink), "Open web interface", theme::button_secondary).on_press(open_web)];
    iced::widget::container(empty_state(
        glyph,
        theme::AMBER,
        format!("{title} - not yet implemented"),
        format!(
            "{summary} This area is not available in YAAMS Commander yet. Use the YAAMS web interface in the meantime."
        ),
        Some(actions.into()),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
