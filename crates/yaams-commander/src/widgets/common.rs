use iced::widget::{Button, Text, button, column, container, row, text, tooltip};
use iced::{Alignment, Element, Length, Padding};
use yaams_api::FlightStatus;

use crate::icons::{Icon, icon};
use crate::theme::{self, Tone};

/// Small uppercase caption used for field labels and section titles.
pub fn caption<'a>(value: impl Into<String>) -> Text<'a> {
    text(value.into().to_uppercase())
        .size(11)
        .font(theme::FONT_BODY_BOLD)
        .color(theme::TEXT_MUTED)
}

pub fn heading<'a>(value: impl Into<String>, size: impl Into<iced::Pixels>) -> Text<'a> {
    text(value.into())
        .size(size)
        .font(theme::FONT_DISPLAY)
        .color(theme::TEXT)
}

pub fn body<'a>(value: impl Into<String>) -> Text<'a> {
    text(value.into()).size(13).color(theme::TEXT_SECONDARY)
}

pub fn mono<'a>(value: impl Into<String>) -> Text<'a> {
    text(value.into()).size(13).font(theme::FONT_MONO)
}

pub fn mono_bold<'a>(value: impl Into<String>) -> Text<'a> {
    text(value.into()).size(13).font(theme::FONT_MONO_BOLD)
}

/// Compact colored chip.
pub fn badge<'a, M: 'a>(label: impl Into<String>, tone: Tone) -> Element<'a, M> {
    container(text(label.into().to_uppercase()).size(10).font(theme::FONT_MONO_BOLD))
        .padding(Padding::from([2, 7]))
        .style(theme::badge(tone))
        .into()
}

pub fn status_tone(status: &FlightStatus) -> Tone {
    match status.id {
        FlightStatus::ACCEPTED => theme::SUCCESS,
        FlightStatus::REJECTED => theme::DANGER,
        FlightStatus::PENDING => theme::WARNING,
        _ => theme::SECONDARY,
    }
}

pub fn status_badge<'a, M: 'a>(status: &FlightStatus) -> Element<'a, M> {
    badge(status.name.clone(), status_tone(status))
}

/// Button with optional leading icon.
pub fn action<'a, M: Clone + 'a>(
    leading: Option<Icon>,
    label: impl Into<String>,
    style: fn(&iced::Theme, button::Status) -> button::Style,
) -> Button<'a, M> {
    let label = text(label.into()).size(13).font(theme::FONT_BODY_BOLD);
    let content: Element<'a, M> = match leading {
        Some(i) => row![icon(i, 14), label].spacing(8).align_y(Alignment::Center).into(),
        None => label.into(),
    };
    button(content).padding(Padding::from([8, 14])).style(style)
}

/// Square icon-only button with a tooltip.
pub fn icon_button<'a, M: Clone + 'a>(glyph: Icon, tip: &'a str, on_press: Option<M>) -> Element<'a, M> {
    let btn = button(container(icon(glyph, 16)).center(Length::Fixed(18.0)))
        .padding(7)
        .style(theme::button_ghost)
        .on_press_maybe(on_press);
    tooltip(
        btn,
        container(text(tip).size(12))
            .padding(Padding::from([4, 8]))
            .style(theme::tooltip),
        tooltip::Position::Bottom,
    )
    .gap(4)
    .into()
}

/// A labelled form field with optional hint and error.
pub fn field<'a, M: 'a>(
    label: &'a str,
    input: impl Into<Element<'a, M>>,
    hint: Option<String>,
    error: Option<&'a str>,
) -> Element<'a, M> {
    let mut col = column![caption(label), input.into()].spacing(6);
    if let Some(error) = error {
        col = col.push(
            row![
                icon(Icon::CircleAlert, 12).color(theme::DANGER.light),
                text(error).size(12).color(theme::DANGER.light)
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        );
    } else if let Some(hint) = hint {
        col = col.push(text(hint).size(12).color(theme::TEXT_MUTED));
    }
    col.width(Length::Fill).into()
}

/// Full-width notice with icon, title and optional trailing action.
pub fn banner<'a, M: 'a>(
    tone: Tone,
    glyph: Icon,
    title: impl Into<String>,
    message: impl Into<String>,
    trailing: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let mut content = row![
        icon(glyph, 16).color(tone.fg()),
        column![
            text(title.into())
                .size(13)
                .font(theme::FONT_BODY_BOLD)
                .color(theme::TEXT),
            text(message.into()).size(12).color(theme::TEXT_SECONDARY),
        ]
        .spacing(2)
        .width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    if let Some(trailing) = trailing {
        content = content.push(trailing);
    }
    container(content)
        .padding(Padding::from([10, 14]))
        .width(Length::Fill)
        .style(theme::tone_box(tone))
        .into()
}

/// Panel with a titled header.
pub fn card<'a, M: 'a>(title: impl Into<String>, glyph: Icon, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(
        column![
            row![
                icon(glyph, 14).color(theme::AMBER),
                caption(title.into()).color(theme::TEXT_SECONDARY)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            content.into(),
        ]
        .spacing(16),
    )
    .padding(18)
    .width(Length::Fill)
    .style(theme::panel)
    .into()
}

/// Label/value pair for detail panels.
pub fn key_value<'a, M: 'a>(label: &'a str, value: impl Into<Element<'a, M>>) -> Element<'a, M> {
    column![caption(label), value.into()]
        .spacing(4)
        .width(Length::Fill)
        .into()
}

/// Centered placeholder for empty, error and not-yet-available states.
pub fn empty_state<'a, M: 'a>(
    glyph: Icon,
    tint: iced::Color,
    title: impl Into<String>,
    message: impl Into<String>,
    actions: Option<Element<'a, M>>,
) -> Element<'a, M> {
    let mut col = column![
        container(icon(glyph, 30).color(tint))
            .center(Length::Fixed(72.0))
            .style(theme::icon_disc(tint)),
        heading(title, 20),
        text(message.into())
            .size(14)
            .color(theme::TEXT_SECONDARY)
            .align_x(iced::alignment::Horizontal::Center),
    ]
    .spacing(14)
    .align_x(Alignment::Center)
    .max_width(460);
    if let Some(actions) = actions {
        col = col.push(container(actions).padding(Padding::ZERO.top(6)));
    }
    container(col).center(Length::Fill).into()
}
