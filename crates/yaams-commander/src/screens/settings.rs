//! Application settings: connection profiles and preferences.

use std::path::Path;

use iced::widget::{button, column, container, pick_list, row, scrollable, space, text};
use iced::{Alignment, Element, Length, Padding};
use uuid::Uuid;

use crate::config::{Config, Profile};
use crate::icons::{Icon, icon};
use crate::theme;
use crate::widgets::{action, badge, body, caption, card, mono};

#[derive(Debug, Clone)]
pub enum Message {
    Activate(Uuid),
    AddProfile,
    UpdateToken(Uuid),
    AskRemove(Uuid),
    CancelRemove,
    ConfirmRemove(Uuid),
    PerPage(u32),
    RevealConfig,
}

pub fn view<'a>(config: &'a Config, config_path: &'a Path, confirm_remove: Option<Uuid>) -> Element<'a, Message> {
    let profiles = column(
        config
            .profiles
            .iter()
            .map(|p| profile_row(p, config.active_profile == Some(p.id), confirm_remove == Some(p.id))),
    )
    .spacing(10);

    let profiles_card = card(
        "Connection profiles",
        Icon::Server,
        column![
            profiles,
            row![action(Some(Icon::Plus), "Add profile", theme::button_secondary).on_press(Message::AddProfile)],
        ]
        .spacing(14),
    );

    let prefs_card = card(
        "Preferences",
        Icon::Settings2,
        column![
            row![
                column![
                    text("Rows per page").size(14),
                    body("Default page size for flight lists")
                ]
                .spacing(2)
                .width(Length::Fill),
                pick_list([25u32, 50, 75, 100], Some(config.ui.per_page), Message::PerPage)
                    .padding(Padding::from([7, 10]))
                    .style(theme::pick_list(false))
                    .menu_style(theme::menu),
            ]
            .align_y(Alignment::Center)
        ]
        .spacing(12),
    );

    let storage_card = card(
        "Storage",
        Icon::FolderOpen,
        column![
            row![
                column![
                    text("Configuration file").size(14),
                    mono(config_path.display().to_string())
                        .size(12)
                        .color(theme::TEXT_SECONDARY),
                ]
                .spacing(4)
                .width(Length::Fill),
                action(Some(Icon::FolderOpen), "Show folder", theme::button_secondary).on_press(Message::RevealConfig),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            body("API tokens are stored in this file. It is only readable by your user account."),
        ]
        .spacing(10),
    );

    let about = card(
        "About",
        Icon::Info,
        column![
            row![
                text("YAAMS Commander").size(14),
                space::horizontal(),
                mono(format!("v{}", env!("CARGO_PKG_VERSION"))).color(theme::TEXT_SECONDARY)
            ],
            body("Desktop operations client for YAAMS - Yet Another Airline Management System."),
        ]
        .spacing(6),
    );

    scrollable(
        column![profiles_card, prefs_card, storage_card, about]
            .spacing(16)
            .max_width(820)
            .padding(Padding::ZERO.right(12)),
    )
    .height(Length::Fill)
    .style(theme::scrollable)
    .into()
}

fn profile_row(profile: &Profile, active: bool, confirming: bool) -> Element<'_, Message> {
    let status: Element<'_, Message> = if active {
        badge("Active", theme::SUCCESS)
    } else {
        space().into()
    };
    let actions: Element<'_, Message> = if confirming {
        row![
            text("Remove this profile?").size(13).color(theme::DANGER.light),
            action(None, "Cancel", theme::button_ghost).on_press(Message::CancelRemove),
            action(Some(Icon::Trash2), "Remove", theme::button_danger).on_press(Message::ConfirmRemove(profile.id)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    } else {
        let mut r = row![].spacing(8).align_y(Alignment::Center);
        if !active {
            r = r.push(
                action(Some(Icon::ArrowRight), "Switch", theme::button_secondary)
                    .on_press(Message::Activate(profile.id)),
            );
        }
        r.push(
            action(Some(Icon::KeyRound), "Update token", theme::button_secondary)
                .on_press(Message::UpdateToken(profile.id)),
        )
        .push(
            button(icon(Icon::Trash2, 15))
                .padding(9)
                .style(theme::button_danger)
                .on_press(Message::AskRemove(profile.id)),
        )
        .into()
    };

    container(
        row![
            container(icon(Icon::Server, 16).color(if active { theme::AMBER } else { theme::TEXT_SECONDARY }))
                .center(Length::Fixed(36.0))
                .style(theme::icon_disc(if active {
                    theme::AMBER
                } else {
                    theme::SECONDARY.strong
                })),
            column![
                row![text(&profile.name).size(14).font(theme::FONT_BODY_BOLD), status]
                    .spacing(10)
                    .align_y(Alignment::Center),
                mono(profile.base_url.clone()).size(12).color(theme::TEXT_SECONDARY),
                caption(format!("Added {}", profile.created_at.format("%Y-%m-%d"))),
            ]
            .spacing(3)
            .width(Length::Fill),
            actions,
        ]
        .spacing(14)
        .align_y(Alignment::Center),
    )
    .padding(14)
    .width(Length::Fill)
    .style(theme::well)
    .into()
}
