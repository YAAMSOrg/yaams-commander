//! Headless UI tests: drive the real `update`/`view` with fixture data and
//! simulate clicks. Set `YAAMS_UI_SNAPSHOTS=<dir>` to also write PNG
//! snapshots of each state for visual review.

use std::time::Duration;

use iced::{Settings, Size};
use iced_test::simulator::Simulator;
use serde_json::json;
use yaams_api::{ApiError, Page as ApiPage};

use crate::app::{App, Bundle, Message, Page};
use crate::config::{Config, Profile, Store};
use crate::screens::{flights, pirep};

const SIZE: Size = Size::new(1440.0, 900.0);

fn settings() -> Settings {
    Settings {
        fonts: crate::assets::FONTS.iter().map(|f| (*f).into()).collect(),
        default_font: crate::theme::FONT_BODY,
        ..Settings::default()
    }
}

fn abilities(all: bool) -> serde_json::Value {
    json!({ "fileFlights": true, "reviewFlights": all, "addAircraft": all, "editAircraft": all,
            "manageMembers": all, "manageInviteCodes": all, "manageNotams": all, "editSettings": all })
}

fn bundle(administer: bool) -> Box<Bundle> {
    let airline = |id: u64, name: &str, prefix: &str, role: &str, all: bool, suspended: bool| {
        json!({ "id": id, "name": name, "prefix": prefix, "icaoCallsign": format!("{prefix}A"),
                "atcCallsign": "EXAMPLE", "unitIsLbs": false, "requirePirepReview": true,
                "locationContinuity": true, "isSuspended": suspended,
                "membership": { "role": role, "isOwner": all, "abilities": abilities(all) } })
    };
    let user = json!({
        "id": 1, "name": "Test Pilot", "email": "pilot@example.com",
        "abilities": { "foundAirline": false, "administerInstance": administer },
        "airlines": [
            airline(1, "Example Virtual Airlines", "EV", "manager", true, false),
            airline(2, "Demo Wings", "DW", "pilot", false, false),
            airline(3, "Suspended Air", "SU", "pilot", false, true),
            { "id": 4, "name": "Not Mine", "prefix": "NM", "membership": null },
        ]
    });
    Box::new(Bundle {
        info: serde_json::from_value(json!({ "name": "Test YAAMS", "version": "1.5.0", "apiVersion": "v1" })).unwrap(),
        user: serde_json::from_value(user).unwrap(),
        networks: serde_json::from_value(json!([{ "id": 1, "name": "Offline" }, { "id": 2, "name": "VATSIM" }]))
            .unwrap(),
    })
}

fn flight(id: u64, number: &str, status: (u8, &str)) -> serde_json::Value {
    json!({ "id": id, "callsign": number, "flightNumber": number, "fullFlightNumber": format!("EV{number}"),
            "fullIcaoCallsign": format!("EVA{number}"), "departureIcao": "EDDF", "arrivalIcao": "EGLL",
            "cruiseAltitude": 36000, "blockOff": "2026-07-11 10:00:00", "blockOn": "2026-07-11 11:30:00",
            "duration": "01:30", "burnedFuel": 4200, "route": "SOVAT UL610 KONAN", "onlineNetwork": 2,
            "status": { "id": status.0, "name": status.1 }, "remarks": "Smooth flight.",
            "rejectionRemarks": if status.0 == 3 { json!("Cruise altitude above aircraft ceiling.") } else { json!(null) },
            "createdAt": "2026-07-11T11:35:00.000000Z", "updatedAt": "2026-07-11T11:40:00.000000Z" })
}

fn flights_page() -> ApiPage<yaams_api::Flight> {
    serde_json::from_value(json!({
        "data": [flight(42, "421", (1, "Pending")), flight(43, "422", (2, "Accepted")), flight(44, "423", (3, "Rejected"))],
        "links": { "first": null, "last": null, "prev": null, "next": null },
        "meta": { "current_page": 1, "from": 1, "to": 3, "per_page": 25, "total": 3, "last_page": 1, "path": "x" }
    }))
    .unwrap()
}

fn fleet() -> Vec<yaams_api::Aircraft> {
    serde_json::from_value(json!([
        { "id": 7, "registration": "D-AIZA", "manufacturer": "Airbus", "model": "A320-200", "currentLoc": "EDDF", "status": "active", "active": true },
        { "id": 8, "registration": "D-ABYC", "manufacturer": "Boeing", "model": "747-8", "currentLoc": "EGLL", "status": "active", "active": true },
    ]))
    .unwrap()
}

/// An app with a connected session on airline 1 and loaded flights.
fn connected_app(dir: &tempfile::TempDir) -> App {
    let store = Store::at(dir.path().join("config.toml"));
    let mut config = Config::default();
    let mut profile = Profile::new("Test".into(), "http://127.0.0.1:9/".into(), "token".into());
    profile.last_airline_id = Some(1);
    config.upsert_profile(profile);
    store.save(&config).unwrap();
    let (mut app, _) = App::with_store(store);
    let _ = app.update(Message::Connected(1, Ok(bundle(true)), Duration::from_millis(42)));
    let _ = app.update(Message::Flights(flights::Message::Loaded(
        1,
        Ok(flights_page()),
        Duration::from_millis(40),
    )));
    let _ = app.update(Message::Flights(flights::Message::CountsLoaded(
        1,
        Ok(flights::Counts {
            pending: 1,
            accepted: 1,
            rejected: 1,
        }),
    )));
    app
}

/// Renders the app, optionally saving a snapshot, and returns the simulator.
fn render<'a>(app: &'a App, name: &str) -> Simulator<'a, Message> {
    let mut ui = Simulator::with_size(settings(), SIZE, app.view());
    if let Some(dir) = std::env::var_os("YAAMS_UI_SNAPSHOTS") {
        let path = std::path::Path::new(&dir).join(name);
        let _ = std::fs::remove_file(path.with_extension("png"));
        ui.snapshot(&app.theme()).unwrap().matches_image(&path).unwrap();
    }
    ui
}

fn click(app: &mut App, text: &str) {
    let messages: Vec<Message> = {
        let mut ui = Simulator::with_size(settings(), SIZE, app.view());
        ui.click(text)
            .unwrap_or_else(|e| panic!("could not click {text:?}: {e}"));
        ui.into_messages().collect()
    };
    for message in messages {
        let _ = app.update(message);
    }
}

#[test]
fn wizard_on_first_run() {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, _) = App::with_store(Store::at(dir.path().join("config.toml")));
    let mut ui = render(&app, "01-wizard-welcome");
    assert!(ui.find("Connect to your virtual airline").is_ok());
    drop(ui);
    click(&mut app, "Get started");
    let mut ui = render(&app, "02-wizard-instance");
    assert!(ui.find("Instance address").is_ok());
    assert!(ui.find("Certificate checks disabled").is_err());
    drop(ui);
    click(&mut app, "Allow self-signed certificates");
    let mut ui = render(&app, "02-wizard-instance-self-signed");
    assert!(ui.find("Certificate checks disabled").is_ok());
}

#[test]
fn flight_log_and_detail() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    let mut ui = render(&app, "03-flights");
    for text in [
        "Flight log",
        "EV421",
        "EV422",
        "Administration",
        "Review queue",
        "File PIREP",
    ] {
        assert!(ui.find(text).is_ok(), "missing {text}");
    }
    // No ability -> no nav entry.
    assert!(ui.find("Found an airline").is_err());
    drop(ui);

    click(&mut app, "EV423");
    let mut ui = render(&app, "04-flight-detail");
    assert!(ui.find("Cruise altitude above aircraft ceiling.").is_ok());
    assert!(ui.find("FL360").is_ok());
}

#[test]
fn pilot_airline_hides_manager_areas() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    app.select_airline(2);
    let mut ui = render(&app, "05-pilot-airline");
    assert!(ui.find("File PIREP").is_ok());
    assert!(ui.find("Review queue").is_err());
    assert!(ui.find("Members").is_err());
}

#[test]
fn suspended_airline_is_read_only() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    app.select_airline(3);
    let mut ui = render(&app, "06-suspended");
    assert!(ui.find("Suspended Air is suspended").is_ok());
    assert!(ui.find("File PIREP").is_err());
}

#[test]
fn pirep_form_validates_and_maps_server_errors() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    let _ = app.update(Message::Navigate(Page::FilePirep));
    let _ = app.update(Message::Pirep(pirep::Message::AircraftLoaded(1, Ok(fleet()))));
    let mut ui = render(&app, "07-pirep-empty");
    assert!(ui.find("FLIGHT STRIP").is_ok());
    assert!(ui.find("Review required").is_ok());
    drop(ui);

    let _ = app.update(Message::Pirep(pirep::Message::Submit));
    let mut ui = render(&app, "08-pirep-errors");
    assert!(ui.find("Check the form").is_ok());
    assert!(ui.find("Select an aircraft").is_ok());
    drop(ui);

    let errors = [(
        "arrival_icao".to_string(),
        vec!["The selected arrival icao is invalid.".to_string()],
    )]
    .into();
    let _ = app.update(Message::Pirep(pirep::Message::Submitted(
        Box::new(Err(ApiError::Validation {
            message: "The given data was invalid.".into(),
            errors,
        })),
        Duration::ZERO,
    )));
    let mut ui = render(&app, "09-pirep-server-errors");
    assert!(ui.find("The selected arrival icao is invalid.").is_ok());
}

#[test]
fn filed_pirep_returns_to_flight_log() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    let _ = app.update(Message::Navigate(Page::FilePirep));
    let filed = serde_json::from_value(flight(99, "999", (1, "Pending"))).unwrap();
    let _ = app.update(Message::Pirep(pirep::Message::Submitted(
        Box::new(Ok(filed)),
        Duration::ZERO,
    )));
    let mut ui = render(&app, "10-pirep-filed");
    assert!(ui.find("PIREP EV999 filed").is_ok());
    assert!(ui.find("Flight log").is_ok());
}

#[test]
fn unimplemented_areas_show_placeholder() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    let _ = app.update(Message::Navigate(Page::ReviewQueue));
    let mut ui = render(&app, "11-not-implemented");
    assert!(ui.find("Review queue - not yet implemented").is_ok());
}

#[test]
fn settings_lists_profiles() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    let _ = app.update(Message::Navigate(Page::Settings));
    let mut ui = render(&app, "12-settings");
    assert!(ui.find("CONNECTION PROFILES").is_ok());
    assert!(ui.find("Update token").is_ok());
}

#[test]
fn rejected_token_shows_banner() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = connected_app(&dir);
    let _ = app.update(Message::Refresh);
    let _ = app.update(Message::Connected(2, Err(ApiError::Unauthorized), Duration::ZERO));
    let mut ui = render(&app, "13-token-rejected");
    assert!(ui.find("API token rejected").is_ok());
}
