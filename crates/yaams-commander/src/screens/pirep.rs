//! PIREP filing form.

use std::collections::BTreeMap;

use chrono::{NaiveDateTime, Utc};
use iced::widget::{button, column, container, pick_list, row, scrollable, space, text, text_input};
use iced::{Alignment, Element, Length, Padding, Task};
use yaams_api::{Aircraft, Airline, ApiError, Client, Flight, NewPirep, OnlineNetwork};

use super::{Event, timed};
use crate::format;
use crate::icons::{Icon, icon};
use crate::theme;
use crate::validation;
use crate::widgets::toast::Kind;
use crate::widgets::{action, banner, caption, card, empty_state, field, mono, mono_bold};

pub struct Context<'a> {
    pub client: &'a Client,
    pub airline: &'a Airline,
    pub networks: &'a [OnlineNetwork],
}

/// Form fields, keyed by their API parameter name for 422 mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Field {
    FlightNumber,
    Callsign,
    Departure,
    Arrival,
    Aircraft,
    Altitude,
    BlockOff,
    BlockOn,
    Fuel,
    Route,
    Network,
    Remarks,
}

impl Field {
    const ALL: [Self; 12] = [
        Self::FlightNumber,
        Self::Callsign,
        Self::Departure,
        Self::Arrival,
        Self::Aircraft,
        Self::Altitude,
        Self::BlockOff,
        Self::BlockOn,
        Self::Fuel,
        Self::Route,
        Self::Network,
        Self::Remarks,
    ];

    fn api_key(self) -> &'static str {
        match self {
            Self::FlightNumber => "flightnumber",
            Self::Callsign => "callsign",
            Self::Departure => "departure_icao",
            Self::Arrival => "arrival_icao",
            Self::Aircraft => "aircraft_id",
            Self::Altitude => "crzalt",
            Self::BlockOff => "blockoff",
            Self::BlockOn => "blockon",
            Self::Fuel => "burned_fuel",
            Self::Route => "route",
            Self::Network => "online_network_id",
            Self::Remarks => "remarks",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    FlightNumber(String),
    Callsign(String),
    Departure(String),
    Arrival(String),
    Aircraft(Box<AircraftChoice>),
    Altitude(String),
    OffDate(String),
    OffTime(String),
    OnDate(String),
    OnTime(String),
    OnNow,
    Fuel(String),
    Route(String),
    Network(OnlineNetwork),
    Remarks(String),
    Submit,
    Clear,
    ReloadAircraft,
    AircraftLoaded(u64, Result<Vec<Aircraft>, ApiError>),
    Submitted(Box<Result<Flight, ApiError>>, std::time::Duration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AircraftChoice(Aircraft);

impl std::fmt::Display for AircraftChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default, Clone)]
struct Inputs {
    flight_number: String,
    callsign: String,
    /// The callsign follows the flight number until edited by hand.
    callsign_edited: bool,
    departure: String,
    arrival: String,
    aircraft: Option<u64>,
    altitude: String,
    off_date: String,
    off_time: String,
    on_date: String,
    on_time: String,
    fuel: String,
    route: String,
    network: Option<u64>,
    remarks: String,
}

enum Fleet {
    Loading,
    Loaded(Vec<Aircraft>),
    Failed(ApiError),
}

pub struct PirepForm {
    airline_id: u64,
    inputs: Inputs,
    fleet: Fleet,
    errors: BTreeMap<Field, String>,
    server_error: Option<String>,
    submitting: bool,
    /// Validate live once the user tried to submit.
    attempted: bool,
}

impl PirepForm {
    pub fn new(ctx: &Context<'_>) -> (Self, Task<Message>) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let inputs = Inputs {
            off_date: today.clone(),
            on_date: today,
            network: ctx.networks.first().map(|n| n.id),
            ..Inputs::default()
        };
        let mut form = Self {
            airline_id: ctx.airline.id,
            inputs,
            fleet: Fleet::Loading,
            errors: BTreeMap::new(),
            server_error: None,
            submitting: false,
            attempted: false,
        };
        let task = form.load_fleet(ctx);
        (form, task)
    }

    pub fn airline_id(&self) -> u64 {
        self.airline_id
    }

    fn load_fleet(&mut self, ctx: &Context<'_>) -> Task<Message> {
        self.fleet = Fleet::Loading;
        let client = ctx.client.clone();
        let airline_id = ctx.airline.id;
        Task::perform(async move { client.active_aircraft(airline_id).await }, move |result| {
            Message::AircraftLoaded(airline_id, result)
        })
    }

    fn aircraft(&self, id: Option<u64>) -> Option<&Aircraft> {
        match &self.fleet {
            Fleet::Loaded(list) => list.iter().find(|a| Some(a.id) == id),
            _ => None,
        }
    }

    pub fn update(&mut self, message: Message, ctx: &Context<'_>, events: &mut Vec<Event>) -> Task<Message> {
        let i = &mut self.inputs;
        match message {
            Message::FlightNumber(v) => {
                let v: String = v.chars().filter(char::is_ascii_digit).take(4).collect();
                if !i.callsign_edited {
                    i.callsign = v.clone();
                }
                i.flight_number = v;
            }
            Message::Callsign(v) => {
                i.callsign = v
                    .to_uppercase()
                    .chars()
                    .filter(char::is_ascii_alphanumeric)
                    .take(6)
                    .collect();
                i.callsign_edited = !i.callsign.is_empty();
            }
            Message::Departure(v) => i.departure = icao_input(&v),
            Message::Arrival(v) => i.arrival = icao_input(&v),
            Message::Aircraft(choice) => {
                let AircraftChoice(aircraft) = *choice;
                let previous_loc = self.aircraft(self.inputs.aircraft).and_then(|a| a.current_loc.clone());
                let i = &mut self.inputs;
                // Follow the aircraft's location while departure is empty or still the
                // previously suggested value.
                if let Some(loc) = &aircraft.current_loc
                    && (i.departure.is_empty() || Some(&i.departure) == previous_loc.as_ref())
                {
                    i.departure = loc.to_uppercase();
                }
                i.aircraft = Some(aircraft.id);
            }
            Message::Altitude(v) => i.altitude = v.to_uppercase(),
            Message::OffDate(v) => {
                if i.on_date == i.off_date {
                    i.on_date = v.clone();
                }
                i.off_date = v;
            }
            Message::OffTime(v) => i.off_time = v,
            Message::OnDate(v) => i.on_date = v,
            Message::OnTime(v) => i.on_time = v,
            Message::OnNow => {
                let now = Utc::now();
                i.on_date = now.format("%Y-%m-%d").to_string();
                i.on_time = now.format("%H:%M").to_string();
            }
            Message::Fuel(v) => i.fuel = v.chars().filter(|c| c.is_ascii_digit() || *c == ',').collect(),
            Message::Route(v) => i.route = v.to_uppercase(),
            Message::Network(n) => i.network = Some(n.id),
            Message::Remarks(v) => i.remarks = v,
            Message::Clear => {
                let network = self.inputs.network;
                let today = Utc::now().format("%Y-%m-%d").to_string();
                self.inputs = Inputs {
                    off_date: today.clone(),
                    on_date: today,
                    network,
                    ..Inputs::default()
                };
                self.errors.clear();
                self.server_error = None;
                self.attempted = false;
                return Task::none();
            }
            Message::ReloadAircraft => return self.load_fleet(ctx),
            Message::AircraftLoaded(airline_id, result) => {
                if airline_id == self.airline_id {
                    match result {
                        Ok(list) => {
                            log::debug!("Loaded {} active aircraft of airline {airline_id}", list.len());
                            self.fleet = Fleet::Loaded(list);
                        }
                        Err(e) => {
                            log::debug!("Loading aircraft of airline {airline_id} failed: {e}");
                            if e.is_unauthorized() {
                                events.push(Event::Unauthorized);
                            }
                            self.fleet = Fleet::Failed(e);
                        }
                    }
                }
                return Task::none();
            }
            Message::Submit => {
                self.attempted = true;
                self.server_error = None;
                let Some(pirep) = self.validate(ctx) else {
                    log::debug!("PIREP not submitted: {} field(s) invalid", self.errors.len());
                    return Task::none();
                };
                log::debug!("Submitting PIREP to airline {}", self.airline_id);
                self.submitting = true;
                let client = ctx.client.clone();
                let airline_id = self.airline_id;
                return Task::perform(
                    async move { timed(client.file_pirep(airline_id, &pirep)).await },
                    |(result, elapsed)| Message::Submitted(Box::new(result), elapsed),
                );
            }
            Message::Submitted(result, elapsed) => {
                self.submitting = false;
                match *result {
                    Ok(flight) => {
                        events.push(Event::Synced(elapsed));
                        events.push(Event::PirepFiled(Box::new(flight)));
                        let network = self.inputs.network;
                        let today = Utc::now().format("%Y-%m-%d").to_string();
                        self.inputs = Inputs {
                            off_date: today.clone(),
                            on_date: today,
                            network,
                            ..Inputs::default()
                        };
                        self.attempted = false;
                        self.errors.clear();
                    }
                    Err(ApiError::Validation { message, errors }) => {
                        log::debug!("PIREP rejected by server validation: {message}");
                        for field in Field::ALL {
                            if let Some(msg) = errors.get(field.api_key()).and_then(|m| m.first()) {
                                self.errors.insert(field, msg.clone());
                            }
                        }
                        self.server_error = Some(message);
                    }
                    Err(ApiError::Unauthorized) => events.push(Event::Unauthorized),
                    Err(e) => {
                        log::debug!("Filing PIREP failed: {e}");
                        events.push(Event::Toast(Kind::Error, "PIREP not filed".into(), e.to_string()));
                        self.server_error = Some(e.to_string());
                    }
                }
                return Task::none();
            }
        }
        if self.attempted {
            self.validate(ctx);
        }
        Task::none()
    }

    /// Validates all inputs, records errors and returns the request on success.
    fn validate(&mut self, ctx: &Context<'_>) -> Option<NewPirep> {
        let i = &self.inputs;
        let mut errors = BTreeMap::new();
        let mut c = Checker(&mut errors);

        let flightnumber = c.check(Field::FlightNumber, validation::flight_number(&i.flight_number));
        let callsign = c.check(Field::Callsign, validation::callsign(&i.callsign));
        let departure = c.check(Field::Departure, validation::icao(&i.departure));
        let arrival = c.check(Field::Arrival, validation::icao(&i.arrival));
        let aircraft = c.check(
            Field::Aircraft,
            self.aircraft(i.aircraft)
                .cloned()
                .ok_or_else(|| "Select an aircraft".to_string()),
        );
        let crzalt = c.check(Field::Altitude, validation::cruise_altitude(&i.altitude));
        let off = c.check(Field::BlockOff, datetime(&i.off_date, &i.off_time));
        let on = c.check(Field::BlockOn, datetime(&i.on_date, &i.on_time));
        let fuel = c.check(Field::Fuel, validation::fuel(&i.fuel));
        let route = c.check(Field::Route, validation::route(&i.route));
        let network = c.check(
            Field::Network,
            i.network
                .filter(|id| ctx.networks.iter().any(|n| n.id == *id))
                .ok_or_else(|| "Select a network".to_string()),
        );
        let remarks = c.check(Field::Remarks, validation::remarks(&i.remarks));
        if let (Some(off), Some(on)) = (off, on) {
            c.check(Field::BlockOn, validation::block_times(off, on));
        }
        if ctx.airline.location_continuity
            && let (Some(aircraft), Some(dep)) = (&aircraft, &departure)
            && let Some(loc) = &aircraft.current_loc
            && !loc.eq_ignore_ascii_case(dep)
        {
            c.check(
                Field::Departure,
                Err::<(), _>(format!("Location continuity: {} is at {loc}", aircraft.registration)),
            );
        }

        self.errors = errors;
        if !self.errors.is_empty() {
            return None;
        }
        Some(NewPirep {
            flightnumber: flightnumber?,
            callsign: callsign?,
            departure_icao: departure?,
            arrival_icao: arrival?,
            aircraft_id: aircraft?.id,
            crzalt: crzalt?,
            blockoff: off?,
            blockon: on?,
            burned_fuel: fuel?,
            route: route?,
            online_network_id: network?,
            remarks: remarks?,
        })
    }

    fn error(&self, field: Field) -> Option<&str> {
        self.errors.get(&field).map(String::as_str)
    }

    pub fn view<'a>(&'a self, ctx: &Context<'a>) -> Element<'a, Message> {
        let airline = ctx.airline;
        let fleet_list: &[Aircraft] = match &self.fleet {
            Fleet::Loaded(list) => list,
            _ => &[],
        };
        if let Fleet::Loaded(list) = &self.fleet
            && list.is_empty()
        {
            return empty_state(
                Icon::Warehouse,
                theme::WARNING.strong,
                "No active aircraft",
                format!(
                    "{} has no active aircraft in its fleet, so no PIREP can be filed. Ask a manager to add or activate an aircraft.",
                    airline.name
                ),
                Some(
                    action(Some(Icon::RefreshCw), "Reload fleet", theme::button_secondary)
                        .on_press(Message::ReloadAircraft)
                        .into(),
                ),
            );
        }

        let i = &self.inputs;
        let input =
            |placeholder: &'a str, value: &'a str, on_input: fn(String) -> Message, field: Field, mono: bool| {
                let mut input = text_input(placeholder, value)
                    .on_input(on_input)
                    .on_submit(Message::Submit)
                    .padding(Padding::from([9, 11]))
                    .size(14)
                    .style(theme::text_input(self.errors.contains_key(&field)));
                if mono {
                    input = input.font(theme::FONT_MONO);
                }
                input
            };

        // --- Flight
        let flight_number_hint =
            (!i.flight_number.is_empty()).then(|| format!("Flight {}{}", airline.prefix, i.flight_number));
        let callsign_hint = (!i.callsign.is_empty() && !airline.icao_callsign.is_empty()).then(|| {
            format!(
                "Radio: {}{}  ({} {})",
                airline.icao_callsign, i.callsign, airline.atc_callsign, i.callsign
            )
        });
        let network_choice = ctx.networks.iter().find(|n| Some(n.id) == i.network).cloned();
        let flight_card = card(
            "Flight",
            Icon::TicketsPlane,
            column![
                row![
                    field(
                        "Flight number",
                        input(
                            "421",
                            &i.flight_number,
                            Message::FlightNumber,
                            Field::FlightNumber,
                            true
                        ),
                        flight_number_hint,
                        self.error(Field::FlightNumber)
                    ),
                    field(
                        "Callsign",
                        input("421", &i.callsign, Message::Callsign, Field::Callsign, true),
                        callsign_hint,
                        self.error(Field::Callsign)
                    ),
                ]
                .spacing(14),
                field(
                    "Online network",
                    pick_list(ctx.networks, network_choice, Message::Network)
                        .placeholder("Select network")
                        .text_size(14)
                        .padding(Padding::from([9, 11]))
                        .width(Length::Fill)
                        .style(theme::pick_list(self.errors.contains_key(&Field::Network)))
                        .menu_style(theme::menu),
                    None,
                    self.error(Field::Network),
                ),
            ]
            .spacing(14),
        );

        // --- Aircraft & route
        let aircraft_input: Element<'a, Message> = match &self.fleet {
            Fleet::Loading => container(text("Loading fleet...").size(13).color(theme::TEXT_MUTED))
                .padding(Padding::from([10, 11]))
                .into(),
            Fleet::Failed(e) => row![
                text(e.to_string())
                    .size(12)
                    .color(theme::DANGER.light)
                    .width(Length::Fill),
                action(Some(Icon::RefreshCw), "Retry", theme::button_secondary).on_press(Message::ReloadAircraft),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
            Fleet::Loaded(_) => {
                let choices: Vec<AircraftChoice> = fleet_list.iter().cloned().map(AircraftChoice).collect();
                let selected = self.aircraft(i.aircraft).cloned().map(AircraftChoice);
                pick_list(choices, selected, |choice| Message::Aircraft(Box::new(choice)))
                    .placeholder("Select aircraft")
                    .text_size(14)
                    .padding(Padding::from([9, 11]))
                    .width(Length::Fill)
                    .font(theme::FONT_MONO)
                    .style(theme::pick_list(self.errors.contains_key(&Field::Aircraft)))
                    .menu_style(theme::menu)
                    .into()
            }
        };
        let altitude_hint = validation::cruise_altitude(&i.altitude)
            .ok()
            .map(|ft| format!("{} - {} ft", format::altitude(ft), format::thousands(ft.into())));
        let mut route_col = column![
            field(
                "Aircraft",
                aircraft_input,
                Some(format!("{} active aircraft", fleet_list.len())),
                self.error(Field::Aircraft)
            ),
            row![
                field(
                    "Departure",
                    input("EDDF", &i.departure, Message::Departure, Field::Departure, true),
                    None,
                    self.error(Field::Departure)
                ),
                container(icon(Icon::ArrowRight, 16).color(theme::TEXT_MUTED)).padding(Padding::ZERO.top(30)),
                field(
                    "Arrival",
                    input("EGLL", &i.arrival, Message::Arrival, Field::Arrival, true),
                    None,
                    self.error(Field::Arrival)
                ),
                field(
                    "Cruise altitude",
                    input("36000 or FL360", &i.altitude, Message::Altitude, Field::Altitude, true),
                    altitude_hint,
                    self.error(Field::Altitude)
                ),
            ]
            .spacing(12),
            field(
                "Route",
                input("SOVAT UL610 KONAN", &i.route, Message::Route, Field::Route, true),
                Some("Use DCT for a direct flight".into()),
                self.error(Field::Route)
            ),
        ]
        .spacing(14);
        if airline.location_continuity {
            route_col = route_col.push(banner(
                theme::INFO,
                Icon::MapPin,
                "Location continuity",
                "This airline requires flights to depart from the aircraft's current location.",
                None,
            ));
        }
        let route_card = card("Aircraft & route", Icon::Route, route_col);

        // --- Times
        let block = match (datetime(&i.off_date, &i.off_time), datetime(&i.on_date, &i.on_time)) {
            (Ok(off), Ok(on)) => validation::block_times(off, on).ok(),
            _ => None,
        };
        let time_row = |label: &'a str,
                        date: &'a str,
                        time: &'a str,
                        on_date: fn(String) -> Message,
                        on_time: fn(String) -> Message,
                        field_id: Field| {
            field(
                label,
                row![
                    input("YYYY-MM-DD", date, on_date, field_id, true).width(Length::FillPortion(3)),
                    input("HH:MM", time, on_time, field_id, true).width(Length::FillPortion(2)),
                ]
                .spacing(8),
                None,
                self.error(field_id),
            )
        };
        let times_card = card(
            "Block times (UTC)",
            Icon::Clock,
            column![
                row![
                    time_row(
                        "Block off",
                        &i.off_date,
                        &i.off_time,
                        Message::OffDate,
                        Message::OffTime,
                        Field::BlockOff
                    ),
                    time_row(
                        "Block on",
                        &i.on_date,
                        &i.on_time,
                        Message::OnDate,
                        Message::OnTime,
                        Field::BlockOn
                    ),
                ]
                .spacing(14),
                row![
                    icon(Icon::Timer, 14).color(theme::TEXT_MUTED),
                    text("Block time").size(13).color(theme::TEXT_SECONDARY),
                    mono_bold(block.map_or_else(|| "--:--".into(), format::hhmm)).color(if block.is_some() {
                        theme::AMBER
                    } else {
                        theme::TEXT_MUTED
                    }),
                    space::horizontal(),
                    action(Some(Icon::Clock), "Block on = now", theme::button_ghost).on_press(Message::OnNow),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(14),
        );

        // --- Fuel & remarks
        let fuel_card = card(
            "Fuel & remarks",
            Icon::Fuel,
            row![
                field(
                    "Fuel burned",
                    row![
                        input("4200", &i.fuel, Message::Fuel, Field::Fuel, true),
                        container(
                            text(airline.weight_unit())
                                .font(theme::FONT_MONO)
                                .size(13)
                                .color(theme::TEXT_SECONDARY)
                        )
                        .padding(Padding::from([9, 4])),
                    ]
                    .align_y(Alignment::Center),
                    None,
                    self.error(Field::Fuel),
                ),
                container(field(
                    "Remarks (optional)",
                    input("Smooth flight.", &i.remarks, Message::Remarks, Field::Remarks, false),
                    Some("Letters, digits, spaces, '.' and '-'".into()),
                    self.error(Field::Remarks),
                ))
                .width(Length::FillPortion(2)),
            ]
            .spacing(14),
        );

        let form = column![flight_card, route_card, times_card, fuel_card]
            .spacing(16)
            .width(Length::FillPortion(3));
        let summary = self.view_summary(ctx, block);

        scrollable(row![form, summary].spacing(16).padding(Padding::ZERO.right(12)))
            .height(Length::Fill)
            .style(theme::scrollable)
            .into()
    }

    /// Right-hand "flight strip" summary with the submit button.
    fn view_summary<'a>(&'a self, ctx: &Context<'a>, block: Option<i64>) -> Element<'a, Message> {
        let airline = ctx.airline;
        let i = &self.inputs;
        let or_dash = |v: &str, placeholder: &'static str| {
            if v.is_empty() {
                placeholder.to_string()
            } else {
                v.to_string()
            }
        };
        let aircraft = self.aircraft(i.aircraft);

        let strip = container(
            column![
                row![
                    text(format!("{}{}", airline.prefix, or_dash(&i.flight_number, "----")))
                        .font(theme::FONT_MONO_BOLD)
                        .size(22)
                        .color(theme::AMBER),
                    space::horizontal(),
                    mono(format!("{}{}", airline.icao_callsign, or_dash(&i.callsign, "--")))
                        .color(theme::TEXT_SECONDARY),
                ]
                .align_y(Alignment::Center),
                row![
                    text(or_dash(&i.departure, "----")).font(theme::FONT_MONO_BOLD).size(20),
                    container(icon(Icon::Plane, 16).color(theme::AMBER)).center_x(Length::Fill),
                    text(or_dash(&i.arrival, "----")).font(theme::FONT_MONO_BOLD).size(20),
                ]
                .align_y(Alignment::Center),
                iced::widget::rule::horizontal(1).style(theme::divider),
                summary_line(
                    "Aircraft",
                    aircraft.map_or_else(|| "-".into(), |a| format!("{} {}", a.registration, a.model))
                ),
                summary_line(
                    "Cruise",
                    validation::cruise_altitude(&i.altitude).map_or_else(|_| "-".into(), format::altitude)
                ),
                summary_line("Block off", or_dash(&i.off_time, "--:--") + "Z"),
                summary_line("Block on", or_dash(&i.on_time, "--:--") + "Z"),
                summary_line("Block time", block.map_or_else(|| "--:--".into(), format::hhmm)),
                summary_line(
                    "Fuel",
                    if i.fuel.is_empty() {
                        "-".into()
                    } else {
                        format!("{} {}", i.fuel, airline.weight_unit())
                    }
                ),
            ]
            .spacing(12),
        )
        .padding(16)
        .width(Length::Fill)
        .style(theme::well);

        let review_note = if airline.require_pirep_review {
            banner(
                theme::INFO,
                Icon::ClipboardCheck,
                "Review required",
                "Your PIREP is filed as pending until a dispatcher or manager accepts it.",
                None,
            )
        } else {
            banner(
                theme::SUCCESS,
                Icon::CircleCheck,
                "Auto-accept",
                "PIREPs for this airline are accepted immediately.",
                None,
            )
        };

        let error_count = self.errors.len();
        let mut col = column![caption("Flight strip"), strip, review_note].spacing(14);
        if let Some(message) = &self.server_error {
            col = col.push(banner(
                theme::DANGER,
                Icon::CircleAlert,
                "The instance rejected the PIREP",
                message.clone(),
                None,
            ));
        } else if self.attempted && error_count > 0 {
            col = col.push(banner(
                theme::DANGER,
                Icon::CircleAlert,
                "Check the form",
                format!(
                    "{error_count} field{} need{} attention.",
                    if error_count == 1 { "" } else { "s" },
                    if error_count == 1 { "s" } else { "" }
                ),
                None,
            ));
        }
        let submit = button(
            row![
                icon(Icon::Send, 15),
                text(if self.submitting { "Filing..." } else { "File PIREP" })
                    .size(14)
                    .font(theme::FONT_BODY_BOLD)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([11, 18]))
        .width(Length::Fill)
        .style(theme::button_primary)
        .on_press_maybe((!self.submitting && matches!(self.fleet, Fleet::Loaded(_))).then_some(Message::Submit));
        col = col.push(
            row![
                submit,
                action(Some(Icon::Eraser), "Clear", theme::button_secondary)
                    .padding(Padding::from([11, 16]))
                    .on_press_maybe((!self.submitting).then_some(Message::Clear)),
            ]
            .spacing(10),
        );

        container(col).width(Length::FillPortion(2)).max_width(380).into()
    }
}

/// Collects validation errors per field.
struct Checker<'a>(&'a mut BTreeMap<Field, String>);

impl Checker<'_> {
    fn check<T>(&mut self, field: Field, result: Result<T, String>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.0.entry(field).or_insert(error);
                None
            }
        }
    }
}

fn summary_line<'a>(label: &'static str, value: String) -> Element<'a, Message> {
    row![
        text(label).size(12).color(theme::TEXT_MUTED),
        space::horizontal(),
        mono(value).size(12)
    ]
    .align_y(Alignment::Center)
    .into()
}

fn icao_input(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphabetic)
        .take(4)
        .collect::<String>()
        .to_uppercase()
}

fn datetime(date: &str, time: &str) -> Result<NaiveDateTime, String> {
    let date = validation::date(date)?;
    let time = validation::time(time)?;
    Ok(date.and_time(time))
}
