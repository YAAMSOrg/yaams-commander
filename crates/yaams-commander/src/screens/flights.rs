//! Flight log: the pilot's own PIREPs with filters, sorting, paging and a
//! detail panel.

use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, pick_list, row, rule, scrollable, space, text, text_input};
use iced::{Alignment, Element, Length, Padding, Task};
use yaams_api::{Airline, ApiError, Client, Flight, FlightQuery, FlightSort, FlightStatusFilter, OnlineNetwork, Page};

use super::{Event, timed};
use crate::format;
use crate::icons::{Icon, icon};
use crate::theme;
use crate::validation;
use crate::widgets::table::{self, Column};
use crate::widgets::toast::Kind;
use crate::widgets::{
    action, badge, banner, caption, empty_state, icon_button, key_value, mono, mono_bold, status_badge,
};

const COLUMNS: [Column<FlightSort>; 10] = [
    Column::new("Flight", 11, Some(FlightSort::FlightNumber)),
    Column::new("Callsign", 11, Some(FlightSort::Callsign)),
    Column::new("Dep", 8, Some(FlightSort::Departure)),
    Column::new("Arr", 8, Some(FlightSort::Arrival)),
    Column::new("Date", 10, Some(FlightSort::BlockOff)),
    Column::new("Off", 8, None),
    Column::new("On", 8, Some(FlightSort::BlockOn)),
    Column::new("Block", 8, Some(FlightSort::Duration)).right(),
    Column::new("Fuel", 11, Some(FlightSort::BurnedFuel)).right(),
    Column::new("Status", 11, Some(FlightSort::Status)),
];

const PER_PAGE_OPTIONS: [u32; 4] = [25, 50, 75, 100];

/// Everything the screen needs from the session.
pub struct Context<'a> {
    pub client: &'a Client,
    pub airline: &'a Airline,
    pub networks: &'a [OnlineNetwork],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub pending: u32,
    pub accepted: u32,
    pub rejected: u32,
}

#[derive(Debug, Clone)]
pub enum Message {
    StatusFilter(FlightStatusFilter),
    DepartureChanged(String),
    ArrivalChanged(String),
    FromChanged(String),
    ToChanged(String),
    Apply,
    Reset,
    Sort(FlightSort),
    Page(u32),
    PerPage(u32),
    Select(u64),
    CloseDetail,
    /// Handled by the shell: open the PIREP form.
    FilePirep,
    Refresh,
    Loaded(u64, Result<Page<Flight>, ApiError>, std::time::Duration),
    CountsLoaded(u64, Result<Counts, ApiError>),
}

#[derive(Debug, Default, Clone)]
struct Filters {
    departure: String,
    arrival: String,
    from: String,
    to: String,
}

#[derive(Debug, Default, Clone)]
struct FilterErrors {
    departure: Option<String>,
    arrival: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

impl FilterErrors {
    fn any(&self) -> bool {
        self.departure.is_some() || self.arrival.is_some() || self.from.is_some() || self.to.is_some()
    }
}

pub struct Flights {
    query: FlightQuery,
    filters: Filters,
    filter_errors: FilterErrors,
    page: Option<Page<Flight>>,
    counts: Option<Counts>,
    loading: bool,
    error: Option<ApiError>,
    selected: Option<u64>,
    /// Flight to select once it appears (after filing).
    pending_select: Option<u64>,
    request: u64,
}

impl Flights {
    pub fn new(per_page: u32) -> Self {
        Self {
            query: FlightQuery {
                per_page,
                ..FlightQuery::default()
            },
            filters: Filters::default(),
            filter_errors: FilterErrors::default(),
            page: None,
            counts: None,
            loading: false,
            error: None,
            selected: None,
            pending_select: None,
            request: 0,
        }
    }

    pub fn set_per_page(&mut self, per_page: u32) {
        self.query.per_page = per_page;
    }

    /// Clears data, e.g. after switching airline.
    pub fn reset(&mut self) {
        let per_page = self.query.per_page;
        *self = Self::new(per_page);
    }

    /// Selects a flight after the next load (used after filing a PIREP).
    pub fn select_after_load(&mut self, id: u64) {
        self.pending_select = Some(id);
        self.selected = Some(id);
    }

    pub fn is_loaded(&self) -> bool {
        self.page.is_some() || self.loading
    }

    /// Reloads the current page and the status counters.
    pub fn load(&mut self, ctx: &Context<'_>) -> Task<Message> {
        self.request += 1;
        self.loading = true;
        let request = self.request;
        let airline_id = ctx.airline.id;
        log::debug!("Loading flights of airline {airline_id}, page {}", self.query.page);

        let client = ctx.client.clone();
        let query = self.query.clone();
        let page = Task::perform(
            async move { timed(client.flights(airline_id, &query)).await },
            move |(result, elapsed)| Message::Loaded(request, result, elapsed),
        );

        let client = ctx.client.clone();
        let counts = Task::perform(async move { load_counts(&client, airline_id).await }, move |result| {
            Message::CountsLoaded(request, result)
        });
        Task::batch([page, counts])
    }

    pub fn update(&mut self, message: Message, ctx: &Context<'_>, events: &mut Vec<Event>) -> Task<Message> {
        match message {
            Message::StatusFilter(status) => {
                self.query.status = status;
                self.query.page = 1;
                return self.load(ctx);
            }
            Message::DepartureChanged(v) => self.filters.departure = v.to_uppercase(),
            Message::ArrivalChanged(v) => self.filters.arrival = v.to_uppercase(),
            Message::FromChanged(v) => self.filters.from = v,
            Message::ToChanged(v) => self.filters.to = v,
            Message::Apply => {
                if self.apply_filters() {
                    self.query.page = 1;
                    return self.load(ctx);
                }
            }
            Message::Reset => {
                self.filters = Filters::default();
                self.filter_errors = FilterErrors::default();
                let per_page = self.query.per_page;
                self.query = FlightQuery {
                    per_page,
                    ..FlightQuery::default()
                };
                return self.load(ctx);
            }
            Message::Sort(sort) => {
                if self.query.sort == sort {
                    self.query.direction = self.query.direction.toggled();
                } else {
                    self.query.sort = sort;
                    self.query.direction = yaams_api::SortDirection::Descending;
                }
                self.query.page = 1;
                return self.load(ctx);
            }
            Message::Page(page) => {
                self.query.page = page.max(1);
                return self.load(ctx);
            }
            Message::PerPage(per_page) => {
                self.query.per_page = per_page;
                self.query.page = 1;
                return self.load(ctx);
            }
            Message::Select(id) => {
                self.selected = if self.selected == Some(id) { None } else { Some(id) };
            }
            Message::CloseDetail => self.selected = None,
            Message::FilePirep => {}
            Message::Refresh => return self.load(ctx),
            Message::Loaded(request, result, elapsed) => {
                if request != self.request {
                    log::debug!("Ignoring stale flights response");
                    return Task::none();
                }
                self.loading = false;
                match result {
                    Ok(page) => {
                        log::debug!(
                            "Loaded {} flight(s), page {} of {} ({} total)",
                            page.data.len(),
                            page.meta.current_page,
                            page.meta.last_page,
                            page.meta.total
                        );
                        events.push(Event::Synced(elapsed));
                        // A page beyond the end (e.g. after filtering) - jump to the last one.
                        if page.data.is_empty() && page.meta.current_page > page.meta.last_page.max(1) {
                            self.query.page = page.meta.last_page.max(1);
                            return self.load(ctx);
                        }
                        if let Some(id) = self.pending_select.take() {
                            self.selected = Some(id);
                        } else if let Some(id) = self.selected
                            && !page.data.iter().any(|f| f.id == id)
                        {
                            self.selected = None;
                        }
                        self.error = None;
                        self.page = Some(page);
                    }
                    Err(e) => {
                        log::debug!("Loading flights failed: {e}");
                        if e.is_unauthorized() {
                            events.push(Event::Unauthorized);
                        } else if self.page.is_some() {
                            events.push(Event::Toast(
                                Kind::Error,
                                "Could not load flights".into(),
                                e.to_string(),
                            ));
                        }
                        self.error = Some(e);
                    }
                }
            }
            Message::CountsLoaded(request, result) => {
                if request == self.request {
                    self.counts = result.ok();
                }
            }
        }
        Task::none()
    }

    /// Validates the filter inputs into the query. Returns false on errors.
    fn apply_filters(&mut self) -> bool {
        let departure = validation::optional_icao(&self.filters.departure);
        let arrival = validation::optional_icao(&self.filters.arrival);
        let from = validation::optional_date(&self.filters.from);
        let to = validation::optional_date(&self.filters.to);
        let mut errors = FilterErrors {
            departure: departure.as_ref().err().cloned(),
            arrival: arrival.as_ref().err().cloned(),
            from: from.as_ref().err().cloned(),
            to: to.as_ref().err().cloned(),
        };
        if let (Ok(Some(from)), Ok(Some(to))) = (&from, &to)
            && to < from
        {
            errors.to = Some("Must not be before From".into());
        }
        let ok = !errors.any();
        self.filter_errors = errors;
        if ok {
            self.query.departure = departure.unwrap_or_default();
            self.query.arrival = arrival.unwrap_or_default();
            self.query.from = from.unwrap_or_default();
            self.query.to = to.unwrap_or_default();
        }
        ok
    }

    pub fn view<'a>(&'a self, ctx: &Context<'a>, can_file: bool) -> Element<'a, Message> {
        let selected = self
            .selected
            .and_then(|id| self.page.as_ref()?.data.iter().find(|f| f.id == id));
        let table = self.view_table(ctx.airline, can_file);
        let body: Element<'a, Message> = match selected {
            Some(flight) => row![table, view_detail(flight, ctx)]
                .spacing(16)
                .height(Length::Fill)
                .into(),
            None => table,
        };
        column![self.view_summary(), self.view_toolbar(), body]
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Ledger summary bar: one ruled strip of status totals that doubles as
    /// the status filter.
    fn view_summary(&self) -> Element<'_, Message> {
        let counts = self.counts;
        let value = |f: fn(Counts) -> u32| counts.map_or_else(|| "-".to_string(), |c| format::thousands(f(c).into()));
        let cell = |label: &'static str, value: String, glyph: Icon, tint: iced::Color, filter: FlightStatusFilter| {
            let active = self.query.status == filter;
            let figure =
                text(value)
                    .font(theme::FONT_MONO_BOLD)
                    .size(20)
                    .color(if active { theme::AMBER } else { theme::TEXT });
            let label = caption(label).color(if active { theme::TEXT } else { theme::TEXT_MUTED });
            button(
                row![icon(glyph, 14).color(tint), label, space::horizontal(), figure]
                    .spacing(10)
                    .height(Length::Fill)
                    .align_y(Alignment::Center),
            )
            .padding(Padding::from([0, 16]))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(theme::ledger_cell(active))
            .on_press(Message::StatusFilter(filter))
            .into()
        };
        let cells: [Element<'_, Message>; 4] = [
            cell(
                "Total PIREPs",
                value(|c| c.pending + c.accepted + c.rejected),
                Icon::Plane,
                theme::AMBER,
                FlightStatusFilter::All,
            ),
            cell(
                "Pending review",
                value(|c| c.pending),
                Icon::Clock,
                theme::WARNING.strong,
                FlightStatusFilter::Pending,
            ),
            cell(
                "Accepted",
                value(|c| c.accepted),
                Icon::CircleCheck,
                theme::SUCCESS.strong,
                FlightStatusFilter::Accepted,
            ),
            cell(
                "Rejected",
                value(|c| c.rejected),
                Icon::CircleX,
                theme::DANGER.strong,
                FlightStatusFilter::Rejected,
            ),
        ];
        let mut strip = row![].height(Length::Fill);
        for (i, cell) in cells.into_iter().enumerate() {
            if i > 0 {
                strip = strip.push(rule::vertical(1).style(theme::divider));
            }
            strip = strip.push(cell);
        }
        container(strip)
            .height(Length::Fixed(52.0))
            .width(Length::Fill)
            .style(theme::panel)
            .clip(true)
            .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let f = &self.filters;
        let e = &self.filter_errors;
        row![
            icon(Icon::PlaneTakeoff, 14).color(theme::TEXT_MUTED),
            filter_input(
                "DEP",
                &f.departure,
                Message::DepartureChanged,
                e.departure.as_deref(),
                72.0
            ),
            icon(Icon::PlaneLanding, 14).color(theme::TEXT_MUTED),
            filter_input("ARR", &f.arrival, Message::ArrivalChanged, e.arrival.as_deref(), 72.0),
            icon(Icon::CalendarDays, 14).color(theme::TEXT_MUTED),
            filter_input(
                "From YYYY-MM-DD",
                &f.from,
                Message::FromChanged,
                e.from.as_deref(),
                140.0
            ),
            text("-").color(theme::TEXT_MUTED),
            filter_input("To YYYY-MM-DD", &f.to, Message::ToChanged, e.to.as_deref(), 140.0),
            action(Some(Icon::Search), "Apply", theme::button_secondary).on_press(Message::Apply),
            icon_button(Icon::FilterX, "Clear filters", Some(Message::Reset)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    }

    fn view_table<'a>(&'a self, airline: &'a Airline, can_file: bool) -> Element<'a, Message> {
        let header = table::header(&COLUMNS, (self.query.sort, self.query.direction), Message::Sort);

        let body: Element<'a, Message> = match (&self.page, &self.error) {
            (None, Some(error)) => empty_state(
                Icon::CloudOff,
                theme::DANGER.strong,
                "Flights could not be loaded",
                error.to_string(),
                Some(
                    action(Some(Icon::RefreshCw), "Try again", theme::button_secondary)
                        .on_press(Message::Refresh)
                        .into(),
                ),
            ),
            (None, None) => empty_state(
                Icon::Radar,
                theme::AMBER,
                "Loading flights",
                "Fetching your flight log from the instance...",
                None,
            ),
            (Some(page), _) if page.data.is_empty() => {
                let filtered = self.query
                    != FlightQuery {
                        per_page: self.query.per_page,
                        ..FlightQuery::default()
                    };
                let (title, message) = if filtered {
                    ("No matching flights", "No flights match the current filters.")
                } else {
                    (
                        "No flights yet",
                        "Your flight log for this airline is empty. File your first PIREP after landing.",
                    )
                };
                let actions: Option<Element<'a, Message>> = if filtered {
                    Some(
                        action(Some(Icon::FilterX), "Clear filters", theme::button_secondary)
                            .on_press(Message::Reset)
                            .into(),
                    )
                } else if can_file {
                    Some(
                        action(Some(Icon::FilePlus), "File a PIREP", theme::button_primary)
                            .on_press(Message::FilePirep)
                            .into(),
                    )
                } else {
                    None
                };
                empty_state(Icon::Inbox, theme::TEXT_MUTED, title, message, actions)
            }
            (Some(page), _) => {
                let rows = page.data.iter().map(|flight| {
                    let selected = self.selected == Some(flight.id);
                    table::data_row(
                        &COLUMNS,
                        flight_cells(flight, airline),
                        selected,
                        Message::Select(flight.id),
                    )
                });
                scrollable(table::ruled(rows))
                    .height(Length::Fill)
                    .style(theme::scrollable)
                    .into()
            }
        };

        let footer = self.view_pagination();
        container(column![header, container(body).height(Length::Fill), footer])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::panel)
            .clip(true)
            .into()
    }

    fn view_pagination(&self) -> Element<'_, Message> {
        let Some(page) = &self.page else { return space().into() };
        let meta = page.meta;
        let current = meta.current_page;
        let last = meta.last_page.max(1);
        let range = match (meta.from, meta.to) {
            (Some(from), Some(to)) => format!("{from}-{to} of {}", format::thousands(meta.total.into())),
            _ => format!("0 of {}", meta.total),
        };
        let nav =
            |glyph, tip, target: u32, enabled: bool| icon_button(glyph, tip, enabled.then_some(Message::Page(target)));

        let status: Element<'_, Message> = if self.loading {
            row![
                icon(Icon::RefreshCw, 12).color(theme::AMBER),
                text("Updating...").size(12).color(theme::TEXT_SECONDARY)
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        } else {
            space().into()
        };

        container(
            row![
                text(range).size(12).font(theme::FONT_MONO).color(theme::TEXT_SECONDARY),
                status,
                space::horizontal(),
                text("Rows").size(12).color(theme::TEXT_MUTED),
                pick_list(PER_PAGE_OPTIONS, Some(self.query.per_page), Message::PerPage)
                    .text_size(12)
                    .padding(Padding::from([4, 8]))
                    .style(theme::pick_list(false))
                    .menu_style(theme::menu),
                nav(Icon::ChevronsLeft, "First page", 1, current > 1),
                nav(
                    Icon::ChevronLeft,
                    "Previous page",
                    current.saturating_sub(1),
                    current > 1
                ),
                text(format!("{current} / {last}"))
                    .size(12)
                    .font(theme::FONT_MONO)
                    .color(theme::TEXT),
                nav(Icon::ChevronRight, "Next page", current + 1, current < last),
                nav(Icon::ChevronsRight, "Last page", last, current < last),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([8, 14]))
        .width(Length::Fill)
        .style(theme::table_footer)
        .into()
    }
}

fn filter_input<'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: fn(String) -> Message,
    error: Option<&'a str>,
    width: f32,
) -> Element<'a, Message> {
    let input = text_input(placeholder, value)
        .on_input(on_input)
        .on_submit(Message::Apply)
        .padding(Padding::from([7, 10]))
        .size(13)
        .font(theme::FONT_MONO)
        .width(Length::Fixed(width))
        .style(theme::text_input(error.is_some()));
    match error {
        Some(e) => iced::widget::tooltip(
            input,
            container(text(e).size(12).color(theme::DANGER.light))
                .padding(Padding::from([4, 8]))
                .style(theme::tooltip),
            iced::widget::tooltip::Position::Bottom,
        )
        .into(),
        None => input.into(),
    }
}

async fn load_counts(client: &Client, airline_id: u64) -> Result<Counts, ApiError> {
    let query = |status| FlightQuery {
        status,
        per_page: 1,
        ..FlightQuery::default()
    };
    let (pending, accepted, rejected) = (
        query(FlightStatusFilter::Pending),
        query(FlightStatusFilter::Accepted),
        query(FlightStatusFilter::Rejected),
    );
    let (p, a, r) = tokio::join!(
        client.flights(airline_id, &pending),
        client.flights(airline_id, &accepted),
        client.flights(airline_id, &rejected),
    );
    Ok(Counts {
        pending: p?.meta.total,
        accepted: a?.meta.total,
        rejected: r?.meta.total,
    })
}

fn flight_cells<'a>(flight: &'a Flight, airline: &Airline) -> Vec<Element<'a, Message>> {
    let off = flight.block_off_time();
    let on = flight.block_on_time();
    let fuel = flight.burned_fuel.map_or_else(
        || "-".to_string(),
        |f| format!("{} {}", format::thousands(f.into()), airline.weight_unit()),
    );
    vec![
        mono_bold(display_or_dash(&flight.full_flight_number))
            .color(theme::AMBER)
            .wrapping(Wrapping::None)
            .into(),
        mono(display_or_dash(&flight.full_icao_callsign))
            .wrapping(Wrapping::None)
            .into(),
        mono_bold(display_or_dash(&flight.departure_icao))
            .wrapping(Wrapping::None)
            .into(),
        mono_bold(display_or_dash(&flight.arrival_icao))
            .wrapping(Wrapping::None)
            .into(),
        mono(format::short_date(off))
            .color(theme::TEXT_SECONDARY)
            .wrapping(Wrapping::None)
            .into(),
        mono(format::clock(off)).wrapping(Wrapping::None).into(),
        mono(format::clock(on)).wrapping(Wrapping::None).into(),
        mono(flight.duration.clone().unwrap_or_else(|| "-".into()))
            .wrapping(Wrapping::None)
            .into(),
        mono(fuel).color(theme::TEXT_SECONDARY).wrapping(Wrapping::None).into(),
        status_badge(&flight.status),
    ]
}

fn display_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".into()
    } else {
        value.to_string()
    }
}

fn view_detail<'a>(flight: &'a Flight, ctx: &Context<'a>) -> Element<'a, Message> {
    let airline = ctx.airline;
    let network = flight
        .online_network
        .and_then(|id| ctx.networks.iter().find(|n| n.id == id))
        .map_or_else(|| "-".to_string(), |n| n.name.clone());

    let header = row![
        column![
            caption("Flight"),
            text(display_or_dash(&flight.full_flight_number))
                .font(theme::FONT_MONO_BOLD)
                .size(24)
                .color(theme::AMBER),
            mono(display_or_dash(&flight.full_icao_callsign)).color(theme::TEXT_SECONDARY),
        ]
        .spacing(2),
        space::horizontal(),
        icon_button(Icon::X, "Close", Some(Message::CloseDetail)),
    ]
    .align_y(Alignment::Start);

    let airport = |code: &'a str, label: &'a str, time: Option<chrono::NaiveDateTime>| {
        column![
            caption(label),
            text(display_or_dash(code)).font(theme::FONT_MONO_BOLD).size(22),
            mono(format::block_time(time)).size(12).color(theme::TEXT_SECONDARY),
        ]
        .spacing(2)
    };
    let route_strip = container(
        row![
            airport(&flight.departure_icao, "Departure", flight.block_off_time()),
            column![
                icon(Icon::Plane, 18).color(theme::AMBER),
                mono(flight.duration.clone().unwrap_or_else(|| "-".into()))
                    .size(12)
                    .color(theme::TEXT_SECONDARY),
            ]
            .spacing(4)
            .align_x(Alignment::Center)
            .width(Length::Fill),
            airport(&flight.arrival_icao, "Arrival", flight.block_on_time()).align_x(Alignment::End),
        ]
        .align_y(Alignment::Center),
    )
    .padding(14)
    .width(Length::Fill)
    .style(theme::well);

    let grid = column![
        row![
            key_value(
                "Cruise",
                mono(flight.cruise_altitude.map_or_else(|| "-".into(), format::altitude))
            ),
            key_value(
                "Fuel burned",
                mono(flight.burned_fuel.map_or_else(
                    || "-".into(),
                    |f| format!("{} {}", format::thousands(f.into()), airline.weight_unit())
                )),
            ),
        ]
        .spacing(12),
        row![
            key_value("Network", text(network).size(13)),
            key_value("Status", row![status_badge(&flight.status)]),
        ]
        .spacing(12),
        row![
            key_value("Filed", mono(format::timestamp(flight.created_at.as_deref())).size(12)),
            key_value(
                "Updated",
                mono(format::timestamp(flight.updated_at.as_deref())).size(12)
            ),
        ]
        .spacing(12),
    ]
    .spacing(14);

    let mut col = column![header, route_strip, grid].spacing(18);

    let route = flight.route.clone().unwrap_or_default();
    col = col.push(key_value(
        "Route",
        container(mono(if route.is_empty() { "-".into() } else { route }).size(12))
            .padding(10)
            .width(Length::Fill)
            .style(theme::well),
    ));
    if let Some(remarks) = flight.remarks.as_ref().filter(|r| !r.is_empty()) {
        col = col.push(key_value(
            "Remarks",
            text(remarks).size(13).color(theme::TEXT_SECONDARY),
        ));
    }
    if flight.status.is_rejected() {
        let reason = flight
            .rejection_remarks
            .clone()
            .filter(|r| !r.is_empty())
            .unwrap_or_else(|| "No reason given.".into());
        col = col.push(banner(theme::DANGER, Icon::CircleX, "Rejected", reason, None));
    } else if flight.status.is_pending() {
        col = col.push(banner(
            theme::WARNING,
            Icon::Clock,
            "Awaiting review",
            "A dispatcher or manager will review this PIREP.",
            None,
        ));
    }
    col = col.push(row![badge(format!("ID {}", flight.id), theme::SECONDARY)]);

    container(scrollable(container(col).padding(18)).style(theme::scrollable))
        .width(Length::Fixed(340.0))
        .height(Length::Fill)
        .style(theme::panel)
        .into()
}
