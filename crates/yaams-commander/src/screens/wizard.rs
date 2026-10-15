//! First-run / add-profile wizard: instance URL, API token, profile name.

use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, space, svg, text, text_input};
use iced::{Alignment, Element, Length, Padding, Task};
use yaams_api::{ApiError, Client, ClientOptions, InstanceInfo, User};

use crate::config::Profile;
use crate::icons::{Icon, icon};
use crate::theme;
use crate::widgets::{action, badge, banner, body, caption, field, heading, mono};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Welcome,
    Instance,
    Token,
    Profile,
}

impl Step {
    const ALL: [Self; 4] = [Self::Welcome, Self::Instance, Self::Token, Self::Profile];

    fn title(self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::Instance => "Instance",
            Self::Token => "API token",
            Self::Profile => "Profile",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Welcome => "What you need",
            Self::Instance => "Locate your YAAMS",
            Self::Token => "Authenticate",
            Self::Profile => "Name and defaults",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    Cancel,
    UrlChanged(String),
    AcceptInvalidCertsToggled(bool),
    InfoLoaded(Result<(String, InstanceInfo), ApiError>),
    TokenChanged(String),
    ToggleToken,
    UserLoaded(Result<User, ApiError>),
    NameChanged(String),
    AirlineSelected(AirlineChoice),
    OpenUrl(String),
}

/// What the wizard asks the shell to do.
pub enum Outcome {
    None,
    Task(Task<Message>),
    Finished(Profile),
    Cancelled,
    OpenUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirlineChoice {
    pub id: u64,
    label: String,
}

impl std::fmt::Display for AirlineChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

pub struct Wizard {
    step: Step,
    url: String,
    base_url: Option<String>,
    accept_invalid_certs: bool,
    info: Option<InstanceInfo>,
    token: String,
    show_token: bool,
    user: Option<User>,
    name: String,
    airline: Option<AirlineChoice>,
    busy: bool,
    error: Option<String>,
    /// Existing profile whose connection is being updated.
    editing: Option<Profile>,
    cancellable: bool,
    notice: Option<String>,
}

impl Wizard {
    /// Wizard for a new profile. `cancellable` is false on first run.
    pub fn new(cancellable: bool, notice: Option<String>) -> Self {
        Self {
            step: Step::Welcome,
            url: String::new(),
            base_url: None,
            accept_invalid_certs: false,
            info: None,
            token: String::new(),
            show_token: false,
            user: None,
            name: String::new(),
            airline: None,
            busy: false,
            error: None,
            editing: None,
            cancellable,
            notice,
        }
    }

    /// Wizard to replace the token (and optionally URL) of an existing profile.
    pub fn update_token(profile: Profile) -> (Self, Task<Message>) {
        let mut wizard = Self::new(true, None);
        wizard.url = profile.base_url.clone();
        wizard.name = profile.name.clone();
        wizard.accept_invalid_certs = profile.accept_invalid_certs;
        wizard.editing = Some(profile);
        wizard.step = Step::Instance;
        let task = wizard.check_url();
        (wizard, task)
    }

    pub fn update(&mut self, message: Message) -> Outcome {
        match message {
            Message::Next => self.next(),
            Message::Back => {
                self.error = None;
                self.busy = false;
                self.step = match self.step {
                    Step::Welcome | Step::Instance => Step::Welcome,
                    Step::Token => Step::Instance,
                    Step::Profile => Step::Token,
                };
                Outcome::None
            }
            Message::Cancel => Outcome::Cancelled,
            Message::UrlChanged(url) => {
                self.url = url;
                self.info = None;
                self.error = None;
                Outcome::None
            }
            Message::AcceptInvalidCertsToggled(accept) => {
                self.accept_invalid_certs = accept;
                self.info = None;
                self.error = None;
                Outcome::None
            }
            Message::InfoLoaded(result) => {
                self.busy = false;
                match result {
                    Ok((base_url, info)) if info.api_version == "v1" => {
                        log::debug!("Wizard: found {} {} at {base_url}", info.name, info.version);
                        self.base_url = Some(base_url);
                        if self.name.is_empty() {
                            self.name = info.name.clone();
                        }
                        self.info = Some(info);
                        if self.editing.is_some() && self.step == Step::Instance {
                            self.step = Step::Token;
                        }
                    }
                    Ok((_, info)) => {
                        log::debug!("Wizard: unsupported API version {}", info.api_version);
                        self.error = Some(format!(
                            "This instance speaks API {}, but YAAMS Commander requires v1.",
                            info.api_version
                        ));
                    }
                    Err(e) => {
                        log::debug!("Wizard: instance check failed: {e}");
                        let mut error = e.to_string();
                        if !self.accept_invalid_certs && error.to_lowercase().contains("certificate") {
                            error.push_str(" If the instance uses a self-signed certificate, allow it below.");
                        }
                        self.error = Some(error);
                    }
                }
                Outcome::None
            }
            Message::TokenChanged(token) => {
                self.token = token;
                self.user = None;
                self.error = None;
                Outcome::None
            }
            Message::ToggleToken => {
                self.show_token = !self.show_token;
                Outcome::None
            }
            Message::UserLoaded(result) => {
                self.busy = false;
                match result {
                    Ok(user) => {
                        log::debug!("Wizard: token belongs to '{}'", user.name);
                        let previous = self.editing.as_ref().and_then(|p| p.last_airline_id);
                        let choices = airline_choices(&user);
                        self.airline = choices
                            .iter()
                            .find(|c| Some(c.id) == previous)
                            .or_else(|| choices.first())
                            .cloned();
                        self.user = Some(user);
                        self.step = Step::Profile;
                    }
                    Err(ApiError::Unauthorized) => {
                        log::debug!("Wizard: token rejected (401)");
                        self.error = Some("The instance rejected this token. Check that it was copied completely and has not been revoked.".into());
                    }
                    Err(e) => {
                        log::debug!("Wizard: token check failed: {e}");
                        self.error = Some(e.to_string());
                    }
                }
                Outcome::None
            }
            Message::NameChanged(name) => {
                self.name = name;
                Outcome::None
            }
            Message::AirlineSelected(choice) => {
                self.airline = Some(choice);
                Outcome::None
            }
            Message::OpenUrl(url) => Outcome::OpenUrl(url),
        }
    }

    fn next(&mut self) -> Outcome {
        if self.busy {
            return Outcome::None;
        }
        self.error = None;
        match self.step {
            Step::Welcome => {
                self.step = Step::Instance;
                Outcome::None
            }
            Step::Instance => {
                if self.info.is_some() {
                    self.step = Step::Token;
                    Outcome::None
                } else {
                    Outcome::Task(self.check_url())
                }
            }
            Step::Token => Outcome::Task(self.verify_token()),
            Step::Profile => {
                let name = self.name.trim();
                if name.is_empty() {
                    self.error = Some("Give this profile a name.".into());
                    return Outcome::None;
                }
                let (Some(base_url), Some(_)) = (&self.base_url, &self.user) else {
                    return Outcome::None;
                };
                let token = self.token.trim().to_string();
                let mut profile = match self.editing.clone() {
                    Some(mut p) => {
                        p.name = name.to_string();
                        p.base_url = base_url.clone();
                        p.token = token;
                        p
                    }
                    None => Profile::new(name.to_string(), base_url.clone(), token),
                };
                profile.accept_invalid_certs = self.accept_invalid_certs;
                profile.last_airline_id = self.airline.as_ref().map(|a| a.id);
                Outcome::Finished(profile)
            }
        }
    }

    fn check_url(&mut self) -> Task<Message> {
        let client = match Client::with_options(&self.url, None, self.client_options()) {
            Ok(client) => client,
            Err(e) => {
                self.error = Some(e.to_string());
                return Task::none();
            }
        };
        self.busy = true;
        let base = client.base_url().to_string();
        log::debug!("Wizard: checking instance at {base}");
        Task::perform(
            async move { client.instance_info().await.map(|info| (base, info)) },
            Message::InfoLoaded,
        )
    }

    fn verify_token(&mut self) -> Task<Message> {
        if self.token.trim().is_empty() {
            self.error = Some("Paste the API token you created in the YAAMS web interface.".into());
            return Task::none();
        }
        let Some(base_url) = &self.base_url else {
            return Task::none();
        };
        let client = match Client::with_options(base_url, Some(self.token.clone()), self.client_options()) {
            Ok(client) => client,
            Err(e) => {
                self.error = Some(e.to_string());
                return Task::none();
            }
        };
        self.busy = true;
        log::debug!("Wizard: verifying token");
        Task::perform(async move { client.current_user().await }, Message::UserLoaded)
    }

    fn client_options(&self) -> ClientOptions {
        ClientOptions {
            accept_invalid_certs: self.accept_invalid_certs,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let card = row![self.view_rail(), self.view_form()].height(Length::Fixed(560.0));
        let card = container(card)
            .width(Length::Fixed(900.0))
            .style(theme::panel_raised)
            .clip(true);
        container(scrollable(container(card).center_x(Length::Fill).padding(32)).style(theme::scrollable))
            .center(Length::Fill)
            .style(theme::app_background)
            .into()
    }

    /// Left brand rail with the step list.
    fn view_rail(&self) -> Element<'_, Message> {
        let logo = row![
            svg(svg::Handle::from_memory(crate::assets::LOGO_ICON))
                .width(44)
                .height(33),
            column![
                text("YAAMS").font(theme::FONT_DISPLAY).size(26).color(theme::TEXT),
                text("COMMANDER")
                    .font(theme::FONT_MONO)
                    .size(10)
                    .color(theme::TEXT_SECONDARY),
            ]
            .spacing(0),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let steps = column(Step::ALL.iter().enumerate().map(|(i, &step)| {
            let done = step < self.step;
            let current = step == self.step;
            let (marker_color, title_color) = if current {
                (theme::AMBER, theme::TEXT)
            } else if done {
                (theme::SUCCESS.strong, theme::TEXT_SECONDARY)
            } else {
                (theme::BORDER_STRONG, theme::TEXT_MUTED)
            };
            let marker: Element<'_, Message> = if done {
                icon(Icon::Check, 13).color(theme::TEXT).into()
            } else {
                text((i + 1).to_string())
                    .font(theme::FONT_MONO_BOLD)
                    .size(12)
                    .color(if current { theme::ON_AMBER } else { theme::TEXT_MUTED })
                    .into()
            };
            let disc = container(marker)
                .center(Length::Fixed(26.0))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(if current || done { marker_color } else { theme::SIDEBAR }.into()),
                    border: iced::Border {
                        color: marker_color,
                        width: 1.5,
                        radius: 999.0.into(),
                    },
                    ..Default::default()
                });
            row![
                disc,
                column![
                    text(step.title())
                        .size(14)
                        .font(theme::FONT_BODY_BOLD)
                        .color(title_color),
                    text(step.subtitle()).size(12).color(theme::TEXT_MUTED),
                ]
                .spacing(1),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        }))
        .spacing(22);

        let footer = text(format!("v{}  ·  YAAMS API v1", env!("CARGO_PKG_VERSION")))
            .font(theme::FONT_MONO)
            .size(11)
            .color(theme::TEXT_MUTED);

        container(column![logo, space().height(40), steps, space::vertical(), footer])
            .padding(32)
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::SIDEBAR.into()),
                border: iced::Border {
                    radius: iced::border::Radius::new(0.0).left(theme::RADIUS + 4.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn view_form(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.step {
            Step::Welcome => self.view_welcome(),
            Step::Instance => self.view_instance(),
            Step::Token => self.view_token(),
            Step::Profile => self.view_profile(),
        };

        let mut col = column![content].spacing(20).height(Length::Fill);
        if let Some(error) = &self.error {
            col = col.push(banner(
                theme::DANGER,
                Icon::CircleAlert,
                "Something went wrong",
                error.clone(),
                None,
            ));
        }
        col = col.push(space::vertical()).push(self.view_footer());
        container(col)
            .padding(Padding::from([36, 40]))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_footer(&self) -> Element<'_, Message> {
        let back: Element<'_, Message> = if self.step > Step::Welcome {
            action(Some(Icon::ChevronLeft), "Back", theme::button_secondary)
                .on_press_maybe((!self.busy).then_some(Message::Back))
                .into()
        } else {
            space().into()
        };
        let cancel: Element<'_, Message> = if self.cancellable {
            action(None, "Cancel", theme::button_ghost)
                .on_press(Message::Cancel)
                .into()
        } else {
            space().into()
        };
        let (label, glyph) = match self.step {
            Step::Welcome => ("Get started", Icon::ArrowRight),
            Step::Instance if self.info.is_some() => ("Continue", Icon::ArrowRight),
            Step::Instance => ("Connect", Icon::Server),
            Step::Token => ("Verify token", Icon::KeyRound),
            Step::Profile if self.editing.is_some() => ("Save profile", Icon::Check),
            Step::Profile => ("Finish setup", Icon::Check),
        };
        let label = if self.busy { "Checking..." } else { label };
        let next = button(
            row![text(label).size(13).font(theme::FONT_BODY_BOLD), icon(glyph, 14)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding(Padding::from([9, 18]))
        .style(theme::button_primary)
        .on_press_maybe((!self.busy).then_some(Message::Next));
        row![back, cancel, space::horizontal(), next]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
    }

    fn view_welcome(&self) -> Element<'_, Message> {
        let item = |glyph, title: &'static str, detail: &'static str| {
            row![
                container(icon(glyph, 16).color(theme::AMBER))
                    .center(Length::Fixed(36.0))
                    .style(theme::icon_disc(theme::AMBER)),
                column![text(title).size(14).font(theme::FONT_BODY_BOLD), body(detail)].spacing(2),
            ]
            .spacing(14)
            .align_y(Alignment::Center)
        };
        let mut col = column![
            heading("Connect to your virtual airline", 26),
            body("YAAMS Commander is the desktop operations client for YAAMS. Set up a connection profile to get started - you can add more instances later."),
        ]
        .spacing(10);
        if let Some(notice) = &self.notice {
            col = col.push(banner(
                theme::WARNING,
                Icon::TriangleAlert,
                "Configuration reset",
                notice.clone(),
                None,
            ));
        }
        column![
            col,
            column![
                item(
                    Icon::Server,
                    "Instance address",
                    "The URL of your YAAMS website, e.g. https://va.example.com"
                ),
                item(
                    Icon::KeyRound,
                    "Personal API token",
                    "Create one in the web UI under Settings → API tokens"
                ),
                item(
                    Icon::Lock,
                    "Stored locally",
                    "Profiles are kept in your user configuration folder, readable only by you"
                ),
            ]
            .spacing(16),
        ]
        .spacing(28)
        .into()
    }

    fn view_instance(&self) -> Element<'_, Message> {
        let input = text_input("https://va.example.com", &self.url)
            .on_input(Message::UrlChanged)
            .on_submit(Message::Next)
            .padding(Padding::from([10, 12]))
            .size(14)
            .font(theme::FONT_MONO)
            .style(theme::text_input(false));

        let mut col = column![
            heading("Instance address", 26),
            body("Enter the address of the YAAMS instance you fly for. Commander checks that it is reachable and speaks the v1 API."),
            field("Instance URL", input, Some("HTTPS is used when no scheme is given".into()), None),
            checkbox(self.accept_invalid_certs)
                .label("Allow self-signed certificates")
                .on_toggle_maybe((!self.busy).then_some(Message::AcceptInvalidCertsToggled))
                .size(16)
                .text_size(13)
                .spacing(10)
                .style(theme::checkbox),
        ]
        .spacing(12);

        if self.accept_invalid_certs {
            col = col.push(banner(
                theme::WARNING,
                Icon::TriangleAlert,
                "Certificate checks disabled",
                "Commander will not verify the identity of this instance. Only use this for test or internal instances on a network you trust - your API token could otherwise be intercepted.",
                None,
            ));
        }

        if let Some(info) = &self.info {
            let mut details = column![
                row![
                    icon(Icon::CircleCheck, 18).color(theme::SUCCESS.light),
                    text(&info.name).size(15).font(theme::FONT_BODY_BOLD),
                    space::horizontal(),
                    badge(format!("YAAMS {}", info.version), theme::SECONDARY),
                    badge(format!("API {}", info.api_version), theme::SUCCESS),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(10);
            if let Some(base) = &self.base_url {
                details = details.push(mono(base.clone()).color(theme::TEXT_SECONDARY));
            }
            let mut links = row![].spacing(14);
            if let Some(email) = &info.support_email {
                links = links.push(
                    row![icon(Icon::Inbox, 12).color(theme::TEXT_MUTED), body(email.clone())]
                        .spacing(6)
                        .align_y(Alignment::Center),
                );
            }
            for (label, url) in [
                ("Imprint", &info.legal.imprint_url),
                ("Privacy", &info.legal.privacy_url),
                ("Terms", &info.legal.terms_url),
            ] {
                if let Some(url) = url {
                    links = links.push(
                        button(text(label).size(12))
                            .padding(0)
                            .style(theme::button_link)
                            .on_press(Message::OpenUrl(url.clone())),
                    );
                }
            }
            details = details.push(links);
            col = col.push(
                container(details)
                    .padding(16)
                    .width(Length::Fill)
                    .style(theme::tone_box(theme::SUCCESS)),
            );
        }
        col.into()
    }

    fn view_token(&self) -> Element<'_, Message> {
        let input = text_input("Paste your token", &self.token)
            .on_input(Message::TokenChanged)
            .on_submit(Message::Next)
            .secure(!self.show_token)
            .padding(Padding::from([10, 12]))
            .size(14)
            .font(theme::FONT_MONO)
            .style(theme::text_input(false));
        let toggle = button(icon(if self.show_token { Icon::EyeOff } else { Icon::Eye }, 16))
            .padding(10)
            .style(theme::button_secondary)
            .on_press(Message::ToggleToken);
        let instance = self.info.as_ref().map_or_else(String::new, |i| i.name.clone());
        let tokens_url = self.base_url.clone().unwrap_or_default();

        column![
            heading("API token", 26),
            body(format!("Commander signs in to {instance} with a personal API token. Create one in the YAAMS web interface and paste it below.")),
            field("Personal API token", row![input, toggle].spacing(8), None, None),
            container(
                row![
                    icon(Icon::Info, 16).color(theme::INFO.light),
                    column![
                        text("Where do I get a token?").size(13).font(theme::FONT_BODY_BOLD),
                        body("Sign in to the web interface, open Settings → API tokens and create a new token. It is shown only once."),
                        button(row![text("Open web interface").size(12), icon(Icon::ExternalLink, 12)].spacing(6).align_y(Alignment::Center))
                            .padding(0)
                            .style(theme::button_link)
                            .on_press(Message::OpenUrl(tokens_url)),
                    ]
                    .spacing(6),
                ]
                .spacing(12),
            )
            .padding(16)
            .width(Length::Fill)
            .style(theme::tone_box(theme::INFO)),
        ]
        .spacing(12)
        .into()
    }

    fn view_profile(&self) -> Element<'_, Message> {
        let Some(user) = &self.user else { return space().into() };
        let name = text_input("e.g. My VA", &self.name)
            .on_input(Message::NameChanged)
            .on_submit(Message::Next)
            .padding(Padding::from([10, 12]))
            .size(14)
            .style(theme::text_input(false));

        let choices = airline_choices(user);
        let airline_field: Element<'_, Message> = if choices.is_empty() {
            banner(
                theme::WARNING,
                Icon::TriangleAlert,
                "No airline memberships",
                "This account is not a member of any airline yet. Join one in the web interface - Commander picks it up on the next refresh.",
                None,
            )
        } else {
            field(
                "Default airline",
                pick_list(choices, self.airline.clone(), Message::AirlineSelected)
                    .text_size(14)
                    .padding(Padding::from([10, 12]))
                    .width(Length::Fill)
                    .style(theme::pick_list(false))
                    .menu_style(theme::menu),
                Some("You can switch airlines at any time from the top bar".into()),
                None,
            )
        };

        let memberships = column(user.airlines.iter().filter(|a| a.is_member()).map(|airline| {
            let role = airline.membership.as_ref().map(|m| m.role.clone()).unwrap_or_default();
            let mut r = row![
                mono_prefix(&airline.prefix),
                text(&airline.name).size(13).width(Length::Fill),
                badge(role, theme::SECONDARY),
            ]
            .spacing(10)
            .align_y(Alignment::Center);
            if airline.is_suspended {
                r = r.push(badge("Suspended", theme::WARNING));
            }
            r.into()
        }))
        .spacing(8);

        column![
            heading("Profile", 26),
            row![
                icon(Icon::CircleCheck, 16).color(theme::SUCCESS.light),
                body(format!("Signed in as {} ({})", user.name, user.email)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            field("Profile name", name, None, None),
            airline_field,
            column![
                caption("Memberships"),
                scrollable(memberships).height(Length::Shrink).style(theme::scrollable)
            ]
            .spacing(8),
        ]
        .spacing(14)
        .into()
    }
}

fn mono_prefix<'a>(prefix: &str) -> Element<'a, Message> {
    container(
        text(prefix.to_string())
            .font(theme::FONT_MONO_BOLD)
            .size(12)
            .color(theme::AMBER),
    )
    .padding(Padding::from([2, 6]))
    .style(theme::accent_badge)
    .into()
}

fn airline_choices(user: &User) -> Vec<AirlineChoice> {
    user.airlines
        .iter()
        .filter(|a| a.is_member())
        .map(|a| AirlineChoice {
            id: a.id,
            label: format!("{}  {}", a.prefix, a.name),
        })
        .collect()
}
