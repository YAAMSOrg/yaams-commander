use std::time::{Duration, Instant};

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

use crate::icons::{Icon, icon};
use crate::theme::{self, Tone};

const LIFETIME: Duration = Duration::from_secs(6);

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: Kind,
    pub title: String,
    pub message: String,
    created: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Success,
    Error,
    Info,
}

impl Kind {
    fn tone(self) -> Tone {
        match self {
            Self::Success => theme::SUCCESS,
            Self::Error => theme::DANGER,
            Self::Info => theme::INFO,
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Success => Icon::CircleCheck,
            Self::Error => Icon::CircleAlert,
            Self::Info => Icon::Info,
        }
    }
}

/// Stack of transient notifications.
#[derive(Debug, Default)]
pub struct Toasts {
    items: Vec<Toast>,
    next_id: u64,
}

impl Toasts {
    pub fn push(&mut self, kind: Kind, title: impl Into<String>, message: impl Into<String>) {
        self.next_id += 1;
        self.items.push(Toast {
            id: self.next_id,
            kind,
            title: title.into(),
            message: message.into(),
            created: Instant::now(),
        });
        // Keep the stack short.
        if self.items.len() > 4 {
            self.items.remove(0);
        }
    }

    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|t| t.id != id);
    }

    /// Drops expired toasts. Errors stay until dismissed.
    pub fn expire(&mut self, now: Instant) {
        self.items
            .retain(|t| t.kind == Kind::Error || now.duration_since(t.created) < LIFETIME);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn view<'a, M: Clone + 'a>(&'a self, on_dismiss: impl Fn(u64) -> M + 'a) -> Element<'a, M> {
        let items = self.items.iter().map(|toast| {
            let tone = toast.kind.tone();
            container(
                row![
                    icon(toast.kind.icon(), 18).color(tone.fg()),
                    column![
                        text(&toast.title).size(13).font(theme::FONT_BODY_BOLD),
                        text(&toast.message).size(12).color(theme::TEXT_SECONDARY),
                    ]
                    .spacing(3)
                    .width(Length::Fill),
                    button(icon(Icon::X, 14))
                        .padding(4)
                        .style(theme::button_ghost)
                        .on_press(on_dismiss(toast.id)),
                ]
                .spacing(12)
                .align_y(Alignment::Start),
            )
            .padding(Padding::from([12, 14]))
            .width(360)
            .style(theme::toast(tone))
            .into()
        });
        column(items).spacing(10).into()
    }
}
