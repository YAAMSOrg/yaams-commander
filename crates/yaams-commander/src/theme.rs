//! YAAMS brand palette, fonts and widget styles.
//!
//! Every widget in the app is styled explicitly through the functions in
//! this module so no default Iced look leaks through.

use iced::border::Radius;
use iced::font::{Family, Weight};
use iced::widget::checkbox as checkbox_widget;
use iced::widget::pick_list as pick_list_widget;
use iced::widget::scrollable as scrollable_widget;
use iced::widget::text_input as text_input_widget;
use iced::widget::{button, container, overlay::menu, rule};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector, color};

// Brand colors (YAAMS palette).
pub const PRIMARY_DARK: Color = color!(0x20242e);
pub const SURFACE: Color = color!(0x282e3a);
pub const AMBER: Color = color!(0xe8a53f);
pub const AMBER_DARK: Color = color!(0x865f31);
pub const AMBER_DEEP: Color = color!(0x574736);

// Derived neutrals for the dark ops UI.
pub const SIDEBAR: Color = color!(0x1a1e26);
pub const SURFACE_RAISED: Color = color!(0x303745);
pub const SURFACE_HOVER: Color = color!(0x343c4b);
pub const BORDER: Color = color!(0x384050);
pub const BORDER_STRONG: Color = color!(0x4a5366);
pub const BORDER_HOVER: Color = color!(0x5a6478);
pub const TEXT: Color = color!(0xf5f6f7);
pub const TEXT_SECONDARY: Color = color!(0xb6bac2);
pub const TEXT_MUTED: Color = color!(0x9097a3);
pub const PLACEHOLDER: Color = color!(0x858d9a);
pub const ON_AMBER: Color = color!(0x1c2430);

/// Status color pair: strong tone (text/icons) and light tone (background).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tone {
    pub strong: Color,
    pub light: Color,
}

pub const SUCCESS: Tone = Tone {
    strong: color!(0x3a8f42),
    light: color!(0xd9f4d9),
};
pub const DANGER: Tone = Tone {
    strong: color!(0xcc272e),
    light: color!(0xffdfda),
};
pub const WARNING: Tone = Tone {
    strong: color!(0xccac16),
    light: color!(0xf9efc9),
};
pub const INFO: Tone = Tone {
    strong: color!(0x2c94b0),
    light: color!(0xd6f0f9),
};
pub const SECONDARY: Tone = Tone {
    strong: color!(0x69737d),
    light: color!(0xe2e5e8),
};

impl Tone {
    /// Readable foreground for the dark UI (the light tone).
    pub fn fg(self) -> Color {
        self.light
    }

    /// Subtle translucent fill for chips and banners on dark surfaces.
    pub fn fill(self) -> Color {
        Color { a: 0.18, ..self.strong }
    }

    pub fn outline(self) -> Color {
        Color { a: 0.55, ..self.strong }
    }
}

// Fonts (embedded, see main.rs): one IBM Plex superfamily. Sans for prose
// and controls, Condensed for captions and headings, Mono for data.
pub const FONT_BODY: Font = Font::with_name("IBM Plex Sans");
/// Emphasis in prose and controls. Plex SemiBold; full Bold is too heavy at 13px.
pub const FONT_BODY_BOLD: Font = Font {
    weight: Weight::Semibold,
    ..FONT_BODY
};
/// Uppercase captions, field labels and table headers.
pub const FONT_CAPTION: Font = Font {
    family: Family::Name("IBM Plex Sans Condensed"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};
/// Page titles, headings and the wordmark.
pub const FONT_DISPLAY: Font = Font {
    family: Family::Name("IBM Plex Sans Condensed"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};
pub const FONT_MONO: Font = Font::with_name("IBM Plex Mono");
pub const FONT_MONO_BOLD: Font = Font {
    weight: Weight::Bold,
    ..FONT_MONO
};

// Corner scale: small and precise, like instrument bezels.
pub const RADIUS_CHIP: f32 = 2.0;
pub const RADIUS: f32 = 4.0;
pub const RADIUS_PANEL: f32 = 6.0;
pub const RADIUS_DIALOG: f32 = 8.0;

pub fn theme() -> Theme {
    Theme::custom(
        "YAAMS",
        iced::theme::Palette {
            background: PRIMARY_DARK,
            text: TEXT,
            primary: AMBER,
            success: SUCCESS.strong,
            warning: WARNING.strong,
            danger: DANGER.strong,
        },
    )
}

fn border(color: Color, width: f32, radius: f32) -> Border {
    Border {
        color,
        width,
        radius: Radius::from(radius),
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

// ---------------------------------------------------------------- containers

pub fn app_background(_: &Theme) -> container::Style {
    container::Style {
        background: Some(PRIMARY_DARK.into()),
        text_color: Some(TEXT),
        ..Default::default()
    }
}

pub fn sidebar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SIDEBAR.into()),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: Radius::from(0.0),
        },
        text_color: Some(TEXT),
        ..Default::default()
    }
}

pub fn topbar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(PRIMARY_DARK.into()),
        text_color: Some(TEXT),
        ..Default::default()
    }
}

pub fn statusbar(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SIDEBAR.into()),
        text_color: Some(TEXT_SECONDARY),
        ..Default::default()
    }
}

pub fn panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: border(BORDER, 1.0, RADIUS_PANEL),
        text_color: Some(TEXT),
        ..Default::default()
    }
}

pub fn panel_raised(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: border(BORDER, 1.0, RADIUS_DIALOG),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        text_color: Some(TEXT),
        ..Default::default()
    }
}

/// Recessed area inside a panel (route strings, code boxes).
pub fn well(_: &Theme) -> container::Style {
    container::Style {
        background: Some(PRIMARY_DARK.into()),
        border: border(BORDER, 1.0, RADIUS),
        text_color: Some(TEXT),
        ..Default::default()
    }
}

pub fn table_header(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE_RAISED.into()),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: Radius::new(0.0).top(RADIUS_PANEL),
        },
        text_color: Some(TEXT_SECONDARY),
        ..Default::default()
    }
}

/// Header band of a titled card: same raised tone as a table header.
pub fn card_header(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE_RAISED.into()),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: Radius::new(0.0).top(RADIUS_PANEL),
        },
        text_color: Some(TEXT_SECONDARY),
        ..Default::default()
    }
}

/// Footer band below a table (range and paging).
pub fn table_footer(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE_RAISED.into()),
        border: Border {
            color: BORDER,
            width: 0.0,
            radius: Radius::new(0.0).bottom(RADIUS_PANEL),
        },
        text_color: Some(TEXT_SECONDARY),
        ..Default::default()
    }
}

/// Left rail of the first-run wizard dialog.
pub fn wizard_rail(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SIDEBAR.into()),
        border: Border {
            radius: Radius::new(0.0).left(RADIUS_DIALOG),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Numbered wizard step marker. Filled once reached.
pub fn step_marker(tint: Color, filled: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(if filled { tint } else { SIDEBAR }.into()),
        border: border(tint, 1.5, 999.0),
        ..Default::default()
    }
}

/// Solid round fill: user avatar, connection dot.
pub fn dot(fill: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(fill.into()),
        border: border(Color::TRANSPARENT, 0.0, 999.0),
        ..Default::default()
    }
}

pub fn tone_box(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(tone.fill().into()),
        border: border(tone.outline(), 1.0, RADIUS),
        text_color: Some(tone.fg()),
        ..Default::default()
    }
}

pub fn badge(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(tone.fill().into()),
        border: border(tone.outline(), 1.0, RADIUS_CHIP),
        text_color: Some(tone.fg()),
        ..Default::default()
    }
}

pub fn accent_badge(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color { a: 0.16, ..AMBER }.into()),
        border: border(Color { a: 0.5, ..AMBER }, 1.0, RADIUS_CHIP),
        text_color: Some(AMBER),
        ..Default::default()
    }
}

pub fn icon_disc(tone: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Color { a: 0.14, ..tone }.into()),
        border: border(Color { a: 0.4, ..tone }, 1.0, 999.0),
        text_color: Some(tone),
        ..Default::default()
    }
}

pub fn tooltip(_: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE_RAISED.into()),
        border: border(BORDER_STRONG, 1.0, RADIUS_CHIP),
        text_color: Some(TEXT),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

pub fn toast(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(mix(SURFACE_RAISED, tone.strong, 0.12).into()),
        border: Border {
            color: tone.outline(),
            width: 1.0,
            radius: Radius::from(RADIUS),
        },
        text_color: Some(TEXT),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

// ---------------------------------------------------------------- buttons

fn button_base(background: Color, text: Color, border_color: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(background)),
        text_color: text,
        border: border(border_color, 1.0, RADIUS),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn button_primary(_: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button_base(AMBER, ON_AMBER, AMBER),
        button::Status::Hovered => button_base(color!(0xf0b456), ON_AMBER, color!(0xf0b456)),
        button::Status::Pressed => button_base(color!(0xcf8f2e), ON_AMBER, AMBER_DARK),
        button::Status::Disabled => button_base(AMBER_DEEP, color!(0x9a8a74), AMBER_DEEP),
    }
}

pub fn button_secondary(_: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: None,
            ..button_base(Color::TRANSPARENT, TEXT, BORDER_STRONG)
        },
        button::Status::Hovered => button_base(SURFACE_HOVER, TEXT, BORDER_HOVER),
        button::Status::Pressed => button_base(SURFACE_RAISED, TEXT, BORDER_STRONG),
        button::Status::Disabled => button::Style {
            background: None,
            ..button_base(Color::TRANSPARENT, TEXT_MUTED, BORDER)
        },
    }
}

pub fn button_danger(_: &Theme, status: button::Status) -> button::Style {
    let d = DANGER.strong;
    match status {
        button::Status::Active => button_base(Color { a: 0.15, ..d }, DANGER.light, Color { a: 0.6, ..d }),
        button::Status::Hovered => button_base(d, TEXT, d),
        button::Status::Pressed => button_base(mix(d, Color::BLACK, 0.2), TEXT, d),
        button::Status::Disabled => button_base(SURFACE, TEXT_MUTED, BORDER),
    }
}

/// Borderless icon/text button for toolbars.
pub fn button_ghost(_: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: TEXT_SECONDARY,
        border: border(Color::TRANSPARENT, 1.0, RADIUS),
        shadow: Shadow::default(),
        snap: true,
    };
    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(SURFACE_HOVER.into()),
            text_color: TEXT,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(SURFACE.into()),
            text_color: TEXT,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: TEXT_MUTED,
            ..base
        },
    }
}

/// Text link style (amber).
pub fn button_link(_: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered => color!(0xf3c27a),
        button::Status::Disabled => TEXT_MUTED,
        _ => AMBER,
    };
    button::Style {
        background: None,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn nav_item(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (background, text_color) = match (active, status) {
            (true, _) => (Some(Color { a: 0.14, ..AMBER }.into()), AMBER),
            (false, button::Status::Hovered) => (Some(SURFACE.into()), TEXT),
            (false, button::Status::Pressed) => (Some(SURFACE_RAISED.into()), TEXT),
            (false, _) => (None, TEXT_SECONDARY),
        };
        button::Style {
            background,
            text_color,
            border: border(Color::TRANSPARENT, 0.0, RADIUS),
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// One column of the ledger summary bar. Active reads like a selected row.
pub fn ledger_cell(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let background = match (active, status) {
            (true, _) => Some(Color { a: 0.13, ..AMBER }.into()),
            (false, button::Status::Hovered) => Some(SURFACE_HOVER.into()),
            (false, button::Status::Pressed) => Some(SURFACE_RAISED.into()),
            (false, _) => None,
        };
        button::Style {
            background,
            text_color: TEXT,
            border: border(
                if active {
                    Color { a: 0.55, ..AMBER }
                } else {
                    Color::TRANSPARENT
                },
                if active { 1.0 } else { 0.0 },
                0.0,
            ),
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Clickable table row. Rows are ruled, not striped.
pub fn table_row(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let background = match (selected, status) {
            (true, _) => Color { a: 0.13, ..AMBER },
            (false, button::Status::Hovered) => SURFACE_HOVER,
            (false, _) => SURFACE,
        };
        button::Style {
            background: Some(background.into()),
            text_color: TEXT,
            border: Border {
                color: if selected {
                    Color { a: 0.55, ..AMBER }
                } else {
                    Color::TRANSPARENT
                },
                width: if selected { 1.0 } else { 0.0 },
                radius: Radius::from(0.0),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

pub fn table_header_cell(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: if matches!(status, button::Status::Hovered) {
            TEXT
        } else {
            TEXT_SECONDARY
        },
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

// ---------------------------------------------------------------- inputs

pub fn text_input(invalid: bool) -> impl Fn(&Theme, text_input_widget::Status) -> text_input_widget::Style {
    move |_, status| {
        let border_color = if invalid {
            DANGER.strong
        } else {
            match status {
                text_input_widget::Status::Focused { .. } => AMBER,
                text_input_widget::Status::Hovered => BORDER_HOVER,
                _ => BORDER_STRONG,
            }
        };
        let background = match status {
            text_input_widget::Status::Disabled => SURFACE,
            _ => PRIMARY_DARK,
        };
        text_input_widget::Style {
            background: background.into(),
            border: border(border_color, 1.0, RADIUS),
            icon: TEXT_MUTED,
            placeholder: PLACEHOLDER,
            value: if matches!(status, text_input_widget::Status::Disabled) {
                TEXT_SECONDARY
            } else {
                TEXT
            },
            selection: Color { a: 0.35, ..AMBER },
        }
    }
}

pub fn pick_list(invalid: bool) -> impl Fn(&Theme, pick_list_widget::Status) -> pick_list_widget::Style {
    move |_, status| {
        let border_color = if invalid {
            DANGER.strong
        } else {
            match status {
                pick_list_widget::Status::Opened { .. } => AMBER,
                pick_list_widget::Status::Hovered => BORDER_HOVER,
                pick_list_widget::Status::Active => BORDER_STRONG,
            }
        };
        pick_list_widget::Style {
            text_color: TEXT,
            placeholder_color: PLACEHOLDER,
            handle_color: TEXT_SECONDARY,
            background: PRIMARY_DARK.into(),
            border: border(border_color, 1.0, RADIUS),
        }
    }
}

pub fn checkbox(_: &Theme, status: checkbox_widget::Status) -> checkbox_widget::Style {
    let (is_checked, hovered, disabled) = match status {
        checkbox_widget::Status::Active { is_checked } => (is_checked, false, false),
        checkbox_widget::Status::Hovered { is_checked } => (is_checked, true, false),
        checkbox_widget::Status::Disabled { is_checked } => (is_checked, false, true),
    };
    let border_color = if is_checked {
        AMBER
    } else if hovered {
        BORDER_HOVER
    } else {
        BORDER_STRONG
    };
    checkbox_widget::Style {
        background: if is_checked { AMBER } else { PRIMARY_DARK }.into(),
        icon_color: ON_AMBER,
        border: border(border_color, 1.0, RADIUS_CHIP),
        text_color: Some(if disabled { TEXT_MUTED } else { TEXT }),
    }
}

/// Pick list styled as a top-bar switcher.
pub fn switcher(_: &Theme, status: pick_list_widget::Status) -> pick_list_widget::Style {
    let border_color = match status {
        pick_list_widget::Status::Opened { .. } => AMBER,
        pick_list_widget::Status::Hovered => BORDER_HOVER,
        pick_list_widget::Status::Active => BORDER,
    };
    pick_list_widget::Style {
        text_color: TEXT,
        placeholder_color: TEXT_MUTED,
        handle_color: AMBER,
        background: SURFACE.into(),
        border: border(border_color, 1.0, RADIUS),
    }
}

pub fn menu(_: &Theme) -> menu::Style {
    menu::Style {
        background: SURFACE_RAISED.into(),
        border: border(BORDER_STRONG, 1.0, RADIUS),
        text_color: TEXT,
        selected_text_color: ON_AMBER,
        selected_background: AMBER.into(),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        },
    }
}

pub fn scrollable(_: &Theme, status: scrollable_widget::Status) -> scrollable_widget::Style {
    let hovered = matches!(
        status,
        scrollable_widget::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } | scrollable_widget::Status::Dragged {
            is_vertical_scrollbar_dragged: true,
            ..
        }
    );
    let rail = scrollable_widget::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable_widget::Scroller {
            background: (if hovered { BORDER_STRONG } else { BORDER }).into(),
            border: border(Color::TRANSPARENT, 0.0, 4.0),
        },
    };
    scrollable_widget::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable_widget::AutoScroll {
            background: SURFACE_RAISED.into(),
            border: border(BORDER_STRONG, 1.0, 999.0),
            shadow: Shadow::default(),
            icon: TEXT,
        },
    }
}

pub fn divider(_: &Theme) -> rule::Style {
    rule::Style {
        color: BORDER,
        radius: Radius::from(0.0),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}
