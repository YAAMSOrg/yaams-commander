//! Application state, message routing and the main window shell
//! (sidebar, top bar, status bar, toasts).

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use iced::widget::{button, column, container, pick_list, row, rule, space, stack, svg, text};
use iced::{Alignment, Element, Length, Padding, Subscription, Task, Theme};
use uuid::Uuid;
use yaams_api::{Airline, ApiError, Client, InstanceInfo, OnlineNetwork, User};

use crate::config::{Config, Profile, Store};
use crate::icons::{Icon, icon};
use crate::screens::{Event, flights, not_implemented, pirep, settings, timed, wizard};
use crate::theme;
use crate::widgets::toast::{Kind, Toasts};
use crate::widgets::{action, badge, banner, caption, empty_state, heading, icon_button};

const SIDEBAR_WIDTH: f32 = 236.0;

/// Global keyboard shortcuts:
/// - Tab / Shift+Tab move focus between inputs (only when no widget used the key).
/// - F5 and Ctrl+R (Cmd+R on macOS) refresh from the API, even while an input
///   has focus.
fn shortcut(event: iced::Event, status: iced::event::Status, _window: iced::window::Id) -> Option<Message> {
    use iced::keyboard::{Event, Key, key::Named};

    let iced::Event::Keyboard(Event::KeyPressed {
        key,
        modifiers,
        physical_key,
        ..
    }) = event
    else {
        return None;
    };
    match key.as_ref() {
        Key::Named(Named::Tab) if status == iced::event::Status::Ignored => Some(if modifiers.shift() {
            Message::FocusPrevious
        } else {
            Message::FocusNext
        }),
        Key::Named(Named::F5) if modifiers.is_empty() => Some(Message::RefreshShortcut),
        _ if modifiers.command()
            && !modifiers.alt()
            && !modifiers.shift()
            && key.to_latin(physical_key) == Some('r') =>
        {
            Some(Message::RefreshShortcut)
        }
        _ => None,
    }
}

pub struct App {
    store: Store,
    config: Config,
    mode: Mode,
    toasts: Toasts,
    now: DateTime<Utc>,
}

enum Mode {
    Wizard {
        wizard: Box<wizard::Wizard>,
        /// Session to return to when the wizard is cancelled.
        previous: Option<Box<Session>>,
    },
    Main(Box<Session>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    /// Tab / Shift+Tab: move keyboard focus to the next / previous input.
    FocusNext,
    FocusPrevious,
    Wizard(wizard::Message),
    Connected(u64, Result<Box<Bundle>, ApiError>, Duration),
    Navigate(Page),
    AirlineSelected(Choice<u64>),
    ProfileSelected(Choice<Uuid>),
    Refresh,
    /// F5 / Ctrl+R / Cmd+R: refresh unless a connection is already running.
    RefreshShortcut,
    Flights(flights::Message),
    Pirep(pirep::Message),
    Settings(settings::Message),
    OpenWeb,
    DismissToast(u64),
    UpdateActiveToken,
}

/// Data loaded when connecting a profile.
#[derive(Debug, Clone)]
pub struct Bundle {
    pub info: InstanceInfo,
    pub user: User,
    pub networks: Vec<OnlineNetwork>,
}

/// Item of a top-bar switcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice<Id> {
    id: Id,
    label: String,
}

impl<Id> std::fmt::Display for Choice<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// Navigation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Flights,
    FilePirep,
    ReviewQueue,
    Fleet,
    Members,
    AirlineSettings,
    Notifications,
    FoundAirline,
    Administration,
    Settings,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Flights => "Flight log",
            Self::FilePirep => "File PIREP",
            Self::ReviewQueue => "Review queue",
            Self::Fleet => "Fleet",
            Self::Members => "Members",
            Self::AirlineSettings => "Airline settings",
            Self::Notifications => "Notifications",
            Self::FoundAirline => "Found an airline",
            Self::Administration => "Administration",
            Self::Settings => "Settings",
        }
    }

    fn nav_label(self) -> &'static str {
        match self {
            Self::Flights => "Flights",
            other => other.title(),
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Flights => Icon::Plane,
            Self::FilePirep => Icon::FilePlus,
            Self::ReviewQueue => Icon::ClipboardCheck,
            Self::Fleet => Icon::Warehouse,
            Self::Members => Icon::Users,
            Self::AirlineSettings => Icon::Settings2,
            Self::Notifications => Icon::Bell,
            Self::FoundAirline => Icon::Building2,
            Self::Administration => Icon::ShieldCheck,
            Self::Settings => Icon::Settings,
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::ReviewQueue => "Accept or reject pending PIREPs of your airline.",
            Self::Fleet => "Browse the airline's aircraft, their status and locations.",
            Self::Members => "Manage member roles and invite codes.",
            Self::AirlineSettings => "Configure the airline's operations settings and announcements.",
            Self::Notifications => "Your notifications about PIREPs, memberships and announcements.",
            Self::FoundAirline => "Found a new virtual airline on this instance.",
            Self::Administration => "Instance administration for Super-Admins.",
            _ => "",
        }
    }

    fn is_implemented(self) -> bool {
        matches!(self, Self::Flights | Self::FilePirep | Self::Settings)
    }
}

/// Connection state of the active profile.
#[derive(Debug, Clone, PartialEq)]
enum Conn {
    Connecting,
    Online,
    Unauthorized,
    Offline(String),
}

struct Session {
    profile: Profile,
    client: Option<Client>,
    generation: u64,
    conn: Conn,
    info: Option<InstanceInfo>,
    user: Option<User>,
    networks: Vec<OnlineNetwork>,
    airline_id: Option<u64>,
    page: Page,
    flights: flights::Flights,
    pirep: Option<pirep::PirepForm>,
    latency: Option<Duration>,
    last_sync: Option<DateTime<Utc>>,
    confirm_remove: Option<Uuid>,
}

impl Session {
    /// Whether a manual refresh (button or shortcut) is possible right now.
    fn can_refresh(&self) -> bool {
        self.client.is_some() && self.conn != Conn::Connecting
    }

    fn new(profile: Profile, per_page: u32) -> Self {
        let client =
            Client::with_options(&profile.base_url, Some(profile.token.clone()), profile.client_options()).ok();
        let conn = if client.is_some() {
            Conn::Connecting
        } else {
            log::debug!(
                "Invalid instance URL in profile '{}': {}",
                profile.name,
                profile.base_url
            );
            Conn::Offline("The instance URL of this profile is invalid.".into())
        };
        Self {
            profile,
            client,
            generation: 0,
            conn,
            info: None,
            user: None,
            networks: Vec::new(),
            airline_id: None,
            page: Page::Flights,
            flights: flights::Flights::new(per_page),
            pirep: None,
            latency: None,
            last_sync: None,
            confirm_remove: None,
        }
    }

    /// (Re)loads instance info, the user with abilities and reference data.
    fn connect(&mut self) -> Task<Message> {
        let Some(client) = self.client.clone() else {
            return Task::none();
        };
        self.generation += 1;
        if self.user.is_none() {
            self.conn = Conn::Connecting;
        }
        let generation = self.generation;
        log::debug!("Syncing instance info, user and online networks (generation {generation})");
        Task::perform(
            timed(async move {
                let (info, user, networks) =
                    tokio::join!(client.instance_info(), client.current_user(), client.online_networks());
                Ok(Box::new(Bundle {
                    info: info?,
                    user: user?,
                    networks: networks?,
                }))
            }),
            move |(result, elapsed)| Message::Connected(generation, result, elapsed),
        )
    }

    fn airline(&self) -> Option<&Airline> {
        let id = self.airline_id?;
        self.user
            .as_ref()?
            .airlines
            .iter()
            .find(|a| a.id == id && a.is_member())
    }

    fn member_airlines(&self) -> impl Iterator<Item = &Airline> {
        self.user
            .iter()
            .flat_map(|u| u.airlines.iter())
            .filter(|a| a.is_member())
    }

    /// Whether a page is available with the current abilities.
    fn can_open(&self, page: Page) -> bool {
        let abilities = self.airline().map(Airline::abilities).unwrap_or_default();
        let instance = self.user.as_ref().map(|u| u.abilities).unwrap_or_default();
        match page {
            Page::Flights | Page::Notifications | Page::Settings => true,
            Page::Fleet => self.airline().is_some(),
            Page::FilePirep => abilities.file_flights,
            Page::ReviewQueue => abilities.review_flights,
            Page::Members => abilities.manage_members || abilities.manage_invite_codes,
            Page::AirlineSettings => abilities.edit_settings || abilities.manage_notams,
            Page::FoundAirline => instance.found_airline,
            Page::Administration => instance.administer_instance,
        }
    }

    fn flights_ctx(&self) -> Option<flights::Context<'_>> {
        Some(flights::Context {
            client: self.client.as_ref()?,
            airline: self.airline()?,
            networks: &self.networks,
        })
    }

    fn pirep_ctx(&self) -> Option<pirep::Context<'_>> {
        Some(pirep::Context {
            client: self.client.as_ref()?,
            airline: self.airline()?,
            networks: &self.networks,
        })
    }

    /// Prepares the data of the current page.
    fn enter_page(&mut self) -> Task<Message> {
        if !self.can_open(self.page) {
            log::debug!(
                "Page {:?} not available with current abilities, showing flights",
                self.page
            );
            self.page = Page::Flights;
        }
        match self.page {
            Page::Flights => {
                let Some(ctx) = self.flights_ctx() else {
                    return Task::none();
                };
                if self.flights.is_loaded() {
                    return Task::none();
                }
                // Borrow juggling: build the task with a fresh context.
                let client = ctx.client.clone();
                let airline = ctx.airline.clone();
                let ctx = flights::Context {
                    client: &client,
                    airline: &airline,
                    networks: &self.networks,
                };
                self.flights.load(&ctx).map(Message::Flights)
            }
            Page::FilePirep => {
                let airline_id = self.airline_id;
                if self.pirep.as_ref().map(pirep::PirepForm::airline_id) == airline_id {
                    return Task::none();
                }
                let Some(ctx) = self.pirep_ctx() else {
                    return Task::none();
                };
                let (form, task) = pirep::PirepForm::new(&ctx);
                self.pirep = Some(form);
                task.map(Message::Pirep)
            }
            _ => Task::none(),
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let store = Store::from_env().unwrap_or_else(|_| Store::at(std::path::PathBuf::from("yaams-commander.toml")));
        Self::with_store(store)
    }

    pub fn with_store(store: Store) -> (Self, Task<Message>) {
        log::debug!("Loading configuration from {}", store.path().display());
        let loaded = store.load();
        if let Some(notice) = &loaded.notice {
            log::debug!("Configuration problem: {notice}");
        }
        let config = loaded.config;
        log::debug!("{} profile(s) configured", config.profiles.len());
        let mut app = Self {
            store,
            mode: Mode::Wizard {
                wizard: Box::new(wizard::Wizard::new(false, loaded.notice)),
                previous: None,
            },
            config,
            toasts: Toasts::default(),
            now: Utc::now(),
        };
        let task = match app.config.active().cloned() {
            Some(profile) => app.start_session(profile),
            None => {
                log::debug!("No active profile, starting the setup wizard");
                Task::none()
            }
        };
        (app, task)
    }

    pub fn title(&self) -> String {
        match &self.mode {
            Mode::Main(session) => format!("YAAMS Commander - {}", session.profile.name),
            Mode::Wizard { .. } => "YAAMS Commander - Setup".into(),
        }
    }

    pub fn theme(&self) -> Theme {
        theme::theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
            iced::event::listen_with(shortcut),
        ])
    }

    /// Switches the active airline (as the top-bar switcher does).
    #[cfg(test)]
    pub fn select_airline(&mut self, id: u64) {
        let _ = self.update(Message::AirlineSelected(Choice {
            id,
            label: String::new(),
        }));
    }

    fn start_session(&mut self, profile: Profile) -> Task<Message> {
        log::debug!(
            "Starting session for profile '{}' at {}{}",
            profile.name,
            profile.base_url,
            if profile.accept_invalid_certs {
                " (TLS certificate verification disabled)"
            } else {
                ""
            }
        );
        let mut session = Session::new(profile, self.config.ui.per_page);
        session.airline_id = session.profile.last_airline_id;
        let task = session.connect();
        self.mode = Mode::Main(Box::new(session));
        task
    }

    fn save_config(&mut self) {
        log::debug!("Saving configuration to {}", self.store.path().display());
        if let Err(e) = self.store.save(&self.config) {
            log::debug!("Saving configuration failed: {e}");
            self.toasts.push(Kind::Error, "Settings not saved", e);
        }
    }

    fn open_url(&mut self, url: &str) {
        log::debug!("Opening {url} in the system handler");
        if let Err(e) = open::that_detached(url) {
            self.toasts.push(Kind::Error, "Could not open browser", e.to_string());
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.now = Utc::now();
                self.toasts.expire(Instant::now());
                Task::none()
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss(id);
                Task::none()
            }
            Message::FocusNext => iced::widget::operation::focus_next(),
            Message::FocusPrevious => iced::widget::operation::focus_previous(),
            Message::Wizard(message) => self.update_wizard(message),
            other => {
                let Mode::Main(session) = &mut self.mode else {
                    return Task::none();
                };
                let mut events = Vec::new();
                let task = Self::update_session(session, &mut self.config, other, &mut events);
                let follow_up = self.process(events);
                Task::batch([task, follow_up])
            }
        }
    }

    fn update_wizard(&mut self, message: wizard::Message) -> Task<Message> {
        let Mode::Wizard { wizard, .. } = &mut self.mode else {
            return Task::none();
        };
        match wizard.update(message) {
            wizard::Outcome::None => Task::none(),
            wizard::Outcome::Task(task) => task.map(Message::Wizard),
            wizard::Outcome::OpenUrl(url) => {
                self.open_url(&url);
                Task::none()
            }
            wizard::Outcome::Cancelled => {
                let Mode::Wizard { previous, .. } = std::mem::replace(
                    &mut self.mode,
                    Mode::Wizard {
                        wizard: Box::new(wizard::Wizard::new(false, None)),
                        previous: None,
                    },
                ) else {
                    return Task::none();
                };
                match previous {
                    Some(session) => {
                        self.mode = Mode::Main(session);
                        Task::none()
                    }
                    None => match self.config.active().cloned() {
                        Some(profile) => self.start_session(profile),
                        None => Task::none(),
                    },
                }
            }
            wizard::Outcome::Finished(profile) => {
                let name = profile.name.clone();
                log::debug!("Setup wizard finished, saving profile '{name}'");
                self.config.upsert_profile(profile.clone());
                self.save_config();
                self.toasts
                    .push(Kind::Success, "Profile saved", format!("Connected to {name}."));
                self.start_session(profile)
            }
        }
    }

    fn update_session(
        session: &mut Session,
        config: &mut Config,
        message: Message,
        events: &mut Vec<Event>,
    ) -> Task<Message> {
        match message {
            Message::Connected(generation, result, elapsed) => {
                if generation != session.generation {
                    log::debug!("Ignoring stale sync result (generation {generation})");
                    return Task::none();
                }
                match result {
                    Ok(bundle) => {
                        events.push(Event::Synced(elapsed));
                        let Bundle { info, user, networks } = *bundle;
                        log::debug!(
                            "Synced in {} ms: {} {}, user '{}' with {} airline(s), {} online network(s)",
                            elapsed.as_millis(),
                            info.name,
                            info.version,
                            user.name,
                            user.airlines.len(),
                            networks.len()
                        );
                        session.info = Some(info);
                        session.networks = networks;
                        let previous = session.user.replace(user);
                        session.conn = Conn::Online;

                        // Keep the airline if still a member, else pick the first membership.
                        if session.airline().is_none() {
                            let first = session.member_airlines().next().map(|a| a.id);
                            log::debug!("Selecting first member airline: {first:?}");
                            session.airline_id = first;
                            session.flights.reset();
                            session.pirep = None;
                        }
                        // Abilities may have changed (e.g. airline suspended).
                        if previous.is_some() && session.pirep.is_some() && !session.can_open(Page::FilePirep) {
                            session.pirep = None;
                        }
                        session.enter_page()
                    }
                    Err(ApiError::Unauthorized) => {
                        log::debug!("Sync failed: token rejected (401)");
                        session.conn = Conn::Unauthorized;
                        Task::none()
                    }
                    Err(e) => {
                        log::debug!("Sync failed, now offline: {e}");
                        session.conn = Conn::Offline(e.to_string());
                        Task::none()
                    }
                }
            }
            Message::Navigate(page) => {
                log::debug!("Navigating to {page:?}");
                session.confirm_remove = None;
                if session.can_open(page) {
                    session.page = page;
                }
                session.enter_page()
            }
            Message::AirlineSelected(choice) => {
                if session.airline_id == Some(choice.id) {
                    return Task::none();
                }
                log::debug!("Switching to airline {}", choice.id);
                session.airline_id = Some(choice.id);
                session.flights.reset();
                session.pirep = None;
                if let Some(profile) = config.profile_mut(session.profile.id) {
                    profile.last_airline_id = Some(choice.id);
                }
                session.profile.last_airline_id = Some(choice.id);
                events.push(Event::SaveConfig);
                session.enter_page()
            }
            Message::ProfileSelected(choice) => {
                events.push(Event::SwitchProfile(choice.id));
                Task::none()
            }
            Message::RefreshShortcut => {
                if !session.can_refresh() {
                    log::debug!("Refresh shortcut ignored: no client or already connecting");
                    return Task::none();
                }
                log::debug!("Refresh via keyboard shortcut");
                Self::update_session(session, config, Message::Refresh, events)
            }
            Message::Refresh => {
                log::debug!("Manual refresh");
                session.flights.reset();
                session.connect()
            }
            Message::Flights(flights::Message::FilePirep) => {
                if session.can_open(Page::FilePirep) {
                    session.page = Page::FilePirep;
                }
                session.enter_page()
            }
            Message::Flights(message) => {
                let Some(ctx) = session.flights_ctx() else {
                    return Task::none();
                };
                let (client, airline) = (ctx.client.clone(), ctx.airline.clone());
                let ctx = flights::Context {
                    client: &client,
                    airline: &airline,
                    networks: &session.networks,
                };
                session.flights.update(message, &ctx, events).map(Message::Flights)
            }
            Message::Pirep(message) => {
                let Some(ctx) = session.pirep_ctx() else {
                    return Task::none();
                };
                let (client, airline) = (ctx.client.clone(), ctx.airline.clone());
                let ctx = pirep::Context {
                    client: &client,
                    airline: &airline,
                    networks: &session.networks,
                };
                let Some(form) = session.pirep.as_mut() else {
                    return Task::none();
                };
                form.update(message, &ctx, events).map(Message::Pirep)
            }
            Message::Settings(message) => {
                match message {
                    settings::Message::Activate(id) => events.push(Event::SwitchProfile(id)),
                    settings::Message::AddProfile => events.push(Event::AddProfile),
                    settings::Message::UpdateToken(id) => events.push(Event::UpdateToken(id)),
                    settings::Message::AskRemove(id) => session.confirm_remove = Some(id),
                    settings::Message::CancelRemove => session.confirm_remove = None,
                    settings::Message::ConfirmRemove(id) => {
                        session.confirm_remove = None;
                        events.push(Event::RemoveProfile(id));
                    }
                    settings::Message::PerPage(per_page) => {
                        config.ui.per_page = per_page;
                        session.flights.set_per_page(per_page);
                        session.flights.reset();
                        events.push(Event::SaveConfig);
                    }
                    settings::Message::RevealConfig => events.push(Event::RevealConfig),
                }
                Task::none()
            }
            Message::OpenWeb => {
                events.push(Event::OpenUrl(session.profile.base_url.clone()));
                Task::none()
            }
            Message::UpdateActiveToken => {
                events.push(Event::UpdateToken(session.profile.id));
                Task::none()
            }
            Message::Tick
            | Message::FocusNext
            | Message::FocusPrevious
            | Message::DismissToast(_)
            | Message::Wizard(_) => Task::none(),
        }
    }

    /// Applies events reported by screens.
    fn process(&mut self, events: Vec<Event>) -> Task<Message> {
        let mut tasks = Vec::new();
        for event in events {
            match event {
                Event::Toast(kind, title, message) => self.toasts.push(kind, title, message),
                Event::Unauthorized => {
                    log::debug!("Token rejected by the instance (401)");
                    if let Mode::Main(session) = &mut self.mode {
                        session.conn = Conn::Unauthorized;
                    }
                }
                Event::Synced(elapsed) => {
                    if let Mode::Main(session) = &mut self.mode {
                        session.latency = Some(elapsed);
                        session.last_sync = Some(Utc::now());
                        if matches!(session.conn, Conn::Offline(_)) {
                            session.conn = Conn::Online;
                        }
                    }
                }
                Event::PirepFiled(flight) => {
                    log::debug!("PIREP {} filed with id {}", flight.full_flight_number, flight.id);
                    let status = flight.status.name.clone();
                    self.toasts.push(
                        Kind::Success,
                        format!("PIREP {} filed", flight.full_flight_number),
                        format!(
                            "{} → {}  ·  Status: {status}",
                            flight.departure_icao, flight.arrival_icao
                        ),
                    );
                    if let Mode::Main(session) = &mut self.mode {
                        session.page = Page::Flights;
                        session.flights.reset();
                        session.flights.select_after_load(flight.id);
                        tasks.push(session.enter_page());
                    }
                }
                Event::SaveConfig => self.save_config(),
                Event::OpenUrl(url) => self.open_url(&url),
                Event::RevealConfig => {
                    let dir = self.store.path().parent().map(std::path::Path::to_path_buf);
                    if let Some(dir) = dir {
                        let _ = std::fs::create_dir_all(&dir);
                        self.open_url(&dir.display().to_string());
                    }
                }
                Event::SwitchProfile(id) => {
                    let current = match &self.mode {
                        Mode::Main(session) => Some(session.profile.id),
                        Mode::Wizard { .. } => None,
                    };
                    if current != Some(id)
                        && let Some(profile) = self.config.profiles.iter().find(|p| p.id == id).cloned()
                    {
                        log::debug!("Switching to profile '{}'", profile.name);
                        self.config.active_profile = Some(id);
                        self.save_config();
                        tasks.push(self.start_session(profile));
                    }
                }
                Event::AddProfile => {
                    log::debug!("Opening the setup wizard to add a profile");
                    let previous = self.take_session();
                    self.mode = Mode::Wizard {
                        wizard: Box::new(wizard::Wizard::new(true, None)),
                        previous,
                    };
                }
                Event::UpdateToken(id) => {
                    if let Some(profile) = self.config.profiles.iter().find(|p| p.id == id).cloned() {
                        log::debug!("Opening the setup wizard to update the token of '{}'", profile.name);
                        let (wizard, task) = wizard::Wizard::update_token(profile);
                        let previous = self.take_session();
                        self.mode = Mode::Wizard {
                            wizard: Box::new(wizard),
                            previous,
                        };
                        tasks.push(task.map(Message::Wizard));
                    }
                }
                Event::RemoveProfile(id) => {
                    let name = self
                        .config
                        .profiles
                        .iter()
                        .find(|p| p.id == id)
                        .map(|p| p.name.clone())
                        .unwrap_or_default();
                    log::debug!("Removing profile '{name}'");
                    self.config.remove_profile(id);
                    self.save_config();
                    self.toasts.push(
                        Kind::Info,
                        "Profile removed",
                        format!("{name} and its token were deleted."),
                    );
                    let was_active = matches!(&self.mode, Mode::Main(s) if s.profile.id == id);
                    if was_active {
                        match self.config.active().cloned() {
                            Some(profile) => tasks.push(self.start_session(profile)),
                            None => {
                                self.mode = Mode::Wizard {
                                    wizard: Box::new(wizard::Wizard::new(false, None)),
                                    previous: None,
                                }
                            }
                        }
                    }
                }
            }
        }
        Task::batch(tasks)
    }

    fn take_session(&mut self) -> Option<Box<Session>> {
        let placeholder = Mode::Wizard {
            wizard: Box::new(wizard::Wizard::new(false, None)),
            previous: None,
        };
        match std::mem::replace(&mut self.mode, placeholder) {
            Mode::Main(session) => Some(session),
            Mode::Wizard { previous, .. } => previous,
        }
    }

    // ------------------------------------------------------------------ view

    pub fn view(&self) -> Element<'_, Message> {
        let content = match &self.mode {
            Mode::Wizard { wizard, .. } => wizard.view().map(Message::Wizard),
            Mode::Main(session) => self.view_main(session),
        };
        if self.toasts.is_empty() {
            return content;
        }
        let toasts = container(self.toasts.view(Message::DismissToast))
            .align_right(Length::Fill)
            .align_bottom(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 20.0,
                bottom: 44.0,
                left: 0.0,
            });
        stack![content, toasts].into()
    }

    fn view_main<'a>(&'a self, session: &'a Session) -> Element<'a, Message> {
        let mut main = column![self.view_topbar(session)].spacing(0);
        main = main.push(rule::horizontal(1).style(theme::divider));
        let mut body = column![].spacing(16).height(Length::Fill);
        for banner in view_banners(session) {
            body = body.push(banner);
        }
        body = body.push(view_page(session, &self.config, self.store.path()));
        main = main.push(container(body).padding(Padding::from([20, 24])).height(Length::Fill));

        let shell = row![
            view_sidebar(session),
            rule::vertical(1).style(theme::divider),
            main.width(Length::Fill)
        ]
        .height(Length::Fill);
        container(column![shell, view_statusbar(session, self.now)])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::app_background)
            .into()
    }

    fn view_topbar<'a>(&'a self, session: &'a Session) -> Element<'a, Message> {
        let airline = session.airline();
        let context = match (airline, &session.info) {
            (Some(a), _) => format!("{}  ·  {}", a.name, a.icao_callsign),
            (None, Some(info)) => info.name.clone(),
            (None, None) => session.profile.name.clone(),
        };
        let title = column![caption(context), heading(session.page.title(), 22)].spacing(2);

        let mut controls = row![].spacing(10).align_y(Alignment::Center);

        let airlines: Vec<Choice<u64>> = session
            .member_airlines()
            .map(|a| Choice {
                id: a.id,
                label: format!("{}  {}", a.prefix, a.name),
            })
            .collect();
        if !airlines.is_empty() {
            let selected = airlines.iter().find(|c| Some(c.id) == session.airline_id).cloned();
            controls = controls.push(
                row![
                    icon(Icon::Plane, 14).color(theme::AMBER),
                    pick_list(airlines, selected, Message::AirlineSelected)
                        .placeholder("Select airline")
                        .text_size(13)
                        .padding(Padding::from([7, 10]))
                        .width(Length::Fixed(240.0))
                        .style(theme::switcher)
                        .menu_style(theme::menu),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
            if let Some(membership) = airline.and_then(|a| a.membership.as_ref()) {
                let role = if membership.is_owner {
                    format!("{} · owner", membership.role)
                } else {
                    membership.role.clone()
                };
                controls = controls.push(badge(role, theme::SECONDARY));
            }
        }

        let profiles: Vec<Choice<Uuid>> = self
            .config
            .profiles
            .iter()
            .map(|p| Choice {
                id: p.id,
                label: p.name.clone(),
            })
            .collect();
        let selected = profiles.iter().find(|c| c.id == session.profile.id).cloned();
        controls = controls.push(
            row![
                icon(Icon::Server, 14).color(theme::TEXT_MUTED),
                pick_list(profiles, selected, Message::ProfileSelected)
                    .text_size(13)
                    .padding(Padding::from([7, 10]))
                    .width(Length::Fixed(190.0))
                    .style(theme::switcher)
                    .menu_style(theme::menu),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );

        controls = controls
            .push(rule::vertical(1).style(theme::divider))
            .push(view_clock(self.now));
        controls = controls.push(icon_button(
            Icon::RefreshCw,
            if cfg!(target_os = "macos") {
                "Refresh (F5 / Cmd+R)"
            } else {
                "Refresh (F5 / Ctrl+R)"
            },
            session.can_refresh().then_some(Message::Refresh),
        ));

        container(row![title, space::horizontal(), controls.height(Length::Fixed(40.0))].align_y(Alignment::Center))
            .padding(Padding::from([12, 24]))
            .width(Length::Fill)
            .style(theme::topbar)
            .into()
    }
}

fn view_clock<'a>(now: DateTime<Utc>) -> Element<'a, Message> {
    column![
        text(now.format("%H:%M:%SZ").to_string())
            .font(theme::FONT_MONO_BOLD)
            .size(17)
            .color(theme::TEXT),
        text(now.format("%a %d %b %Y").to_string().to_uppercase())
            .font(theme::FONT_MONO)
            .size(10)
            .color(theme::TEXT_MUTED),
    ]
    .spacing(0)
    .align_x(Alignment::End)
    .width(Length::Fixed(118.0))
    .into()
}

fn view_sidebar(session: &Session) -> Element<'_, Message> {
    let brand = row![
        svg(svg::Handle::from_memory(crate::assets::LOGO_ICON))
            .width(36)
            .height(27),
        column![
            text("YAAMS").font(theme::FONT_DISPLAY).size(21).color(theme::TEXT),
            text("COMMANDER")
                .font(theme::FONT_MONO)
                .size(9)
                .color(theme::TEXT_SECONDARY),
        ]
        .spacing(0),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let item = |page: Page| -> Option<Element<'_, Message>> {
        if !session.can_open(page) {
            return None;
        }
        let active = session.page == page;
        let mut content = row![
            icon(page.icon(), 16),
            text(page.nav_label()).size(13).font(if active {
                theme::FONT_BODY_BOLD
            } else {
                theme::FONT_BODY
            }),
            space::horizontal(),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        if !page.is_implemented() {
            content = content.push(text("SOON").font(theme::FONT_MONO).size(9).color(theme::TEXT_MUTED));
        }
        Some(
            button(content)
                .padding(Padding::from([9, 12]))
                .width(Length::Fill)
                .style(theme::nav_item(active))
                .on_press(Message::Navigate(page))
                .into(),
        )
    };

    let section = |title: &'static str, pages: &[Page]| -> Option<Element<'_, Message>> {
        let items: Vec<_> = pages.iter().filter_map(|&p| item(p)).collect();
        if items.is_empty() {
            return None;
        }
        Some(
            column![
                container(caption(title)).padding(Padding::from([0, 12])),
                column(items).spacing(2)
            ]
            .spacing(8)
            .into(),
        )
    };

    let mut nav = column![].spacing(22);
    for (title, pages) in [
        ("Operations", &[Page::Flights, Page::FilePirep, Page::ReviewQueue][..]),
        ("Airline", &[Page::Fleet, Page::Members, Page::AirlineSettings][..]),
        (
            "Instance",
            &[Page::Notifications, Page::FoundAirline, Page::Administration][..],
        ),
    ] {
        if let Some(section) = section(title, pages) {
            nav = nav.push(section);
        }
    }

    let user_card: Element<'_, Message> = match &session.user {
        Some(user) => row![
            container(
                text(initials(&user.name))
                    .font(theme::FONT_BODY_BOLD)
                    .size(12)
                    .color(theme::ON_AMBER)
            )
            .center(Length::Fixed(32.0))
            .style(|_: &Theme| container::Style {
                background: Some(theme::AMBER.into()),
                border: iced::Border {
                    radius: 999.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            column![
                text(&user.name).size(13).font(theme::FONT_BODY_BOLD),
                text(&user.email).size(11).color(theme::TEXT_MUTED),
            ]
            .spacing(1),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
        None => space().into(),
    };

    container(
        column![
            container(brand).padding(Padding::from([4, 12])),
            space().height(26),
            nav,
            space::vertical(),
            item(Page::Settings).unwrap_or_else(|| space().into()),
            rule::horizontal(1).style(theme::divider),
            container(user_card).padding(Padding::from([4, 8])),
        ]
        .spacing(10),
    )
    .padding(Padding::from([20, 12]))
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .style(theme::sidebar)
    .into()
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn view_banners(session: &Session) -> Vec<Element<'_, Message>> {
    let mut banners = Vec::new();
    match &session.conn {
        Conn::Unauthorized => banners.push(banner(
            theme::DANGER,
            Icon::KeyRound,
            "API token rejected",
            "The instance no longer accepts the token of this profile. It may have been revoked or expired.",
            Some(
                action(Some(Icon::KeyRound), "Update token", theme::button_primary)
                    .on_press(Message::UpdateActiveToken)
                    .into(),
            ),
        )),
        Conn::Offline(reason) => banners.push(banner(
            theme::WARNING,
            Icon::WifiOff,
            "Instance unreachable",
            reason.clone(),
            Some(
                action(Some(Icon::RefreshCw), "Retry", theme::button_secondary)
                    .on_press(Message::Refresh)
                    .into(),
            ),
        )),
        Conn::Connecting | Conn::Online => {}
    }
    if let Some(airline) = session.airline()
        && airline.is_suspended
    {
        banners.push(banner(
            theme::WARNING,
            Icon::Lock,
            format!("{} is suspended", airline.name),
            "The instance administrator suspended this airline. It is read-only - no PIREPs can be filed.",
            None,
        ));
    }
    banners
}

fn view_page<'a>(session: &'a Session, config: &'a Config, config_path: &'a std::path::Path) -> Element<'a, Message> {
    let page = session.page;
    if page == Page::Settings {
        return settings::view(config, config_path, session.confirm_remove).map(Message::Settings);
    }
    if !page.is_implemented() {
        return not_implemented::view(page.title(), page.icon(), page.summary(), Message::OpenWeb);
    }
    if session.user.is_none() {
        return match &session.conn {
            Conn::Connecting => empty_state(
                Icon::Radar,
                theme::AMBER,
                "Connecting",
                format!("Contacting {}...", session.profile.base_url),
                None,
            ),
            Conn::Unauthorized => empty_state(
                Icon::KeyRound,
                theme::DANGER.strong,
                "Not signed in",
                "Update the API token to continue.",
                None,
            ),
            Conn::Offline(_) | Conn::Online => empty_state(
                Icon::CloudOff,
                theme::WARNING.strong,
                "No connection",
                "Commander could not load your account from the instance.",
                None,
            ),
        };
    }
    if session.airline().is_none() {
        return empty_state(
            Icon::Building2,
            theme::TEXT_MUTED,
            "No airline membership",
            "Your account is not a member of any airline on this instance. Join an airline in the web interface, then refresh.",
            Some(
                action(Some(Icon::ExternalLink), "Open web interface", theme::button_secondary)
                    .on_press(Message::OpenWeb)
                    .into(),
            ),
        );
    }
    match page {
        Page::Flights => match session.flights_ctx() {
            Some(ctx) => {
                let can_file = session.can_open(Page::FilePirep);
                let ctx: flights::Context<'a> = ctx;
                session.flights.view(&ctx, can_file).map(Message::Flights)
            }
            None => space().into(),
        },
        Page::FilePirep => match (session.pirep_ctx(), &session.pirep) {
            (Some(ctx), Some(form)) => form.view(&ctx).map(Message::Pirep),
            _ => space().into(),
        },
        _ => space().into(),
    }
}

fn view_statusbar(session: &Session, now: DateTime<Utc>) -> Element<'_, Message> {
    let (color, label) = match &session.conn {
        Conn::Online => (theme::SUCCESS.strong, "Online".to_string()),
        Conn::Connecting => (theme::INFO.strong, "Connecting".to_string()),
        Conn::Unauthorized => (theme::DANGER.strong, "Token rejected".to_string()),
        Conn::Offline(_) => (theme::WARNING.strong, "Offline".to_string()),
    };
    let dot = container(space().width(8).height(8)).style(move |_: &Theme| container::Style {
        background: Some(color.into()),
        border: iced::Border {
            radius: 999.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });
    let small = |value: String| text(value).font(theme::FONT_MONO).size(11).color(theme::TEXT_SECONDARY);
    let sep = || text("|").size(11).color(theme::BORDER_STRONG);

    let mut left = row![dot, small(label)].spacing(8).align_y(Alignment::Center);
    if let Some(info) = &session.info {
        left = left.push(sep()).push(small(format!(
            "{}  ·  YAAMS {}  ·  API {}",
            info.name, info.version, info.api_version
        )));
    }
    if let Some(latency) = session.latency {
        left = left.push(sep()).push(small(format!("{} ms", latency.as_millis())));
    }
    if let Some(sync) = session.last_sync {
        let ago = (now - sync).num_seconds().max(0);
        let label = if ago < 60 {
            format!("synced {ago}s ago")
        } else {
            format!("synced {}Z", sync.format("%H:%M"))
        };
        left = left.push(sep()).push(small(label));
    }
    let right = row![
        small(session.profile.base_url.clone()),
        sep(),
        small(format!("v{}", env!("CARGO_PKG_VERSION")))
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    container(row![left, space::horizontal(), right].align_y(Alignment::Center))
        .padding(Padding::from([5, 14]))
        .width(Length::Fill)
        .style(theme::statusbar)
        .into()
}

#[cfg(test)]
mod tests {
    use iced::event::Status;
    use iced::keyboard::key::{Code, Named, Physical};
    use iced::keyboard::{Event, Key, Location, Modifiers};

    use super::{Message, shortcut};

    fn press(key: Key, code: Code, modifiers: Modifiers, status: Status) -> Option<Message> {
        let event = iced::Event::Keyboard(Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: Physical::Code(code),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        });
        shortcut(event, status, iced::window::Id::unique())
    }

    #[test]
    fn keyboard_shortcuts() {
        let r = || Key::Character("r".into());
        let refresh = |m| matches!(m, Some(Message::RefreshShortcut));

        assert!(refresh(press(
            Key::Named(Named::F5),
            Code::F5,
            Modifiers::empty(),
            Status::Ignored
        )));
        assert!(refresh(press(r(), Code::KeyR, Modifiers::COMMAND, Status::Ignored)));
        // Refresh also works while a text input has focus.
        assert!(refresh(press(r(), Code::KeyR, Modifiers::COMMAND, Status::Captured)));
        assert!(press(r(), Code::KeyR, Modifiers::empty(), Status::Ignored).is_none());
        assert!(press(r(), Code::KeyR, Modifiers::COMMAND | Modifiers::SHIFT, Status::Ignored).is_none());

        let tab = || Key::Named(Named::Tab);
        assert!(matches!(
            press(tab(), Code::Tab, Modifiers::empty(), Status::Ignored),
            Some(Message::FocusNext)
        ));
        assert!(matches!(
            press(tab(), Code::Tab, Modifiers::SHIFT, Status::Ignored),
            Some(Message::FocusPrevious)
        ));
        assert!(press(tab(), Code::Tab, Modifiers::empty(), Status::Captured).is_none());
    }
}
