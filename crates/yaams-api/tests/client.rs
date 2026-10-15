use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use yaams_api::{ApiError, Client, FlightQuery, FlightStatusFilter};

fn airline_json(abilities: serde_json::Value, suspended: bool) -> serde_json::Value {
    json!({
        "id": 1, "name": "Example Virtual Airlines", "prefix": "EV",
        "icaoCallsign": "EVA", "atcCallsign": "EXAMPLE", "unitIsLbs": false,
        "requirePirepReview": true, "locationContinuity": false, "isSuspended": suspended,
        "membership": { "role": "pilot", "isOwner": false, "abilities": abilities },
        "createdAt": "2026-01-01T00:00:00.000000Z", "updatedAt": "2026-01-01T00:00:00.000000Z"
    })
}

fn flight_json(id: u64) -> serde_json::Value {
    json!({
        "id": id, "callsign": "421", "flightNumber": 421, "fullFlightNumber": "EV421",
        "fullIcaoCallsign": "EVA421", "departureIcao": "EDDF", "arrivalIcao": "EGLL",
        "cruiseAltitude": 36000, "blockOff": "2026-07-11 10:00:00",
        "blockOn": "2026-07-11 11:30:00", "duration": "01:30", "burnedFuel": 4200,
        "route": "SOVAT UL610 KONAN", "onlineNetwork": 1,
        "status": { "id": 2, "name": "Accepted" }, "remarks": null, "rejectionRemarks": null,
        "createdAt": "2026-07-11T11:35:00.000000Z", "updatedAt": "2026-07-11T11:40:00.000000Z"
    })
}

fn page_json(data: Vec<serde_json::Value>, page: u32, last: u32) -> serde_json::Value {
    json!({
        "data": data,
        "links": { "first": null, "last": null, "prev": null, "next": null },
        "meta": { "current_page": page, "from": 1, "to": 1, "per_page": 100,
                  "total": last, "last_page": last, "path": "x" }
    })
}

async fn client(server: &MockServer) -> Client {
    Client::new(&server.uri(), Some("secret-token".into())).unwrap()
}

#[tokio::test]
async fn sends_bearer_token_and_parses_user() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/user"))
        .and(header("authorization", "Bearer secret-token"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": 1, "name": "Homer Simpson", "email": "homer@test.com",
                "abilities": { "foundAirline": false, "administerInstance": true },
                "airlines": [airline_json(json!({ "fileFlights": true, "someFutureAbility": true }), false)]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let user = client(&server).await.current_user().await.unwrap();
    assert_eq!(user.name, "Homer Simpson");
    assert!(user.abilities.administer_instance);
    let airline = &user.airlines[0];
    let abilities = airline.abilities();
    assert!(abilities.file_flights);
    // Missing keys default to false.
    assert!(!abilities.review_flights);
    assert!(!abilities.edit_settings);
}

#[tokio::test]
async fn suspended_airline_has_no_abilities() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/airlines/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": airline_json(json!({ "fileFlights": true }), true)
        })))
        .mount(&server)
        .await;
    let airline = client(&server).await.airline(1).await.unwrap();
    assert!(!airline.abilities().file_flights);
}

#[tokio::test]
async fn instance_info_works_without_token_and_with_api_suffix() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "name": "Dev YAAMS", "version": "1.5.0", "apiVersion": "v1",
                      "supportEmail": null, "features": { "registration": true },
                      "legal": { "imprintUrl": null, "privacyUrl": null, "termsUrl": null } }
        })))
        .mount(&server)
        .await;
    let client = Client::new(&format!("{}/api/v1/", server.uri()), None).unwrap();
    let info = client.instance_info().await.unwrap();
    assert_eq!(info.api_version, "v1");
    assert!(!info.features.user_airline_creation);
}

#[tokio::test]
async fn flights_send_filters_and_parse_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/airlines/3/flights"))
        .and(query_param("status", "accepted"))
        .and(query_param("sort", "-block_off"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_json(vec![flight_json(42)], 1, 1)))
        .mount(&server)
        .await;
    let query = FlightQuery {
        status: FlightStatusFilter::Accepted,
        ..FlightQuery::default()
    };
    let page = client(&server).await.flights(3, &query).await.unwrap();
    assert_eq!(page.data.len(), 1);
    let flight = &page.data[0];
    assert_eq!(flight.full_flight_number, "EV421");
    assert!(flight.status.is_accepted());
    assert!(flight.block_off_time().is_some());
    assert!(!page.has_next());
}

#[tokio::test]
async fn active_aircraft_collects_all_pages() {
    let server = MockServer::start().await;
    for page in 1..=2u32 {
        Mock::given(method("GET"))
            .and(path("/api/v1/airlines/1/aircraft"))
            .and(query_param("status", "active"))
            .and(query_param("page", page.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                vec![json!({ "id": page, "registration": format!("D-AAA{page}"),
                             "manufacturer": "Airbus", "model": "A320", "currentLoc": "EDDF",
                             "status": "active", "active": true })],
                page,
                2,
            )))
            .expect(1)
            .mount(&server)
            .await;
    }
    let aircraft = client(&server).await.active_aircraft(1).await.unwrap();
    assert_eq!(aircraft.len(), 2);
    assert_eq!(aircraft[1].registration, "D-AAA2");
}

#[tokio::test]
async fn file_pirep_posts_body_and_maps_validation_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/airlines/1/flights"))
        .and(body_json(json!({
            "flightnumber": 421, "callsign": "421", "departure_icao": "EDDF",
            "arrival_icao": "EGLL", "aircraft_id": 7, "crzalt": 36000,
            "blockoff": "2026-07-11 10:00:00", "blockon": "2026-07-11 11:30:00",
            "burned_fuel": 4200, "route": "DCT", "online_network_id": 1
        })))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "message": "The departure icao field is invalid.",
            "errors": { "departure_icao": ["The selected departure icao is invalid."] }
        })))
        .mount(&server)
        .await;
    let fmt = yaams_api::BLOCK_TIME_FORMAT;
    let pirep = yaams_api::NewPirep {
        flightnumber: 421,
        callsign: "421".into(),
        departure_icao: "EDDF".into(),
        arrival_icao: "EGLL".into(),
        aircraft_id: 7,
        crzalt: 36000,
        blockoff: chrono::NaiveDateTime::parse_from_str("2026-07-11 10:00:00", fmt).unwrap(),
        blockon: chrono::NaiveDateTime::parse_from_str("2026-07-11 11:30:00", fmt).unwrap(),
        burned_fuel: 4200,
        route: "DCT".into(),
        online_network_id: 1,
        remarks: None,
    };
    let err = client(&server).await.file_pirep(1, &pirep).await.unwrap_err();
    assert_eq!(
        err.field_error("departure_icao"),
        Some("The selected departure icao is invalid.")
    );
}

#[tokio::test]
async fn maps_error_statuses() {
    let server = MockServer::start().await;
    Mock::given(path("/api/v1/user"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({ "message": "Unauthenticated." })))
        .mount(&server)
        .await;
    Mock::given(path("/api/v1/airlines/9"))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(json!({ "message": "You are not a member of this airline." })),
        )
        .mount(&server)
        .await;
    Mock::given(path("/api/v1/airlines/8"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({ "message": "No query results" })))
        .mount(&server)
        .await;
    Mock::given(path("/api/v1/online-networks"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<!DOCTYPE html><html></html>"))
        .mount(&server)
        .await;

    let client = client(&server).await;
    assert!(client.current_user().await.unwrap_err().is_unauthorized());
    assert!(matches!(client.airline(9).await.unwrap_err(), ApiError::Forbidden(m) if m.contains("not a member")));
    assert!(matches!(client.airline(8).await.unwrap_err(), ApiError::NotFound(_)));
    assert!(matches!(client.online_networks().await.unwrap_err(), ApiError::Decode(m) if m.contains("HTML")));
}

#[tokio::test]
async fn network_errors_are_reported() {
    let client = Client::new("http://127.0.0.1:9", Some("t".into())).unwrap();
    assert!(matches!(
        client.instance_info().await.unwrap_err(),
        ApiError::Network(_)
    ));
}
