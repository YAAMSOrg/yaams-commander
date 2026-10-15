//! Read-only checks against a real instance. Run with
//! `YAAMS_URL=... YAAMS_TOKEN=... cargo test -p yaams-api -- --ignored`.

use yaams_api::{Client, FlightQuery};

fn live_client() -> Client {
    let url = std::env::var("YAAMS_URL").expect("YAAMS_URL not set");
    let token = std::env::var("YAAMS_TOKEN").ok();
    Client::new(&url, token).unwrap()
}

#[tokio::test]
#[ignore = "needs a live instance"]
async fn live_read_only_endpoints() {
    let client = live_client();
    let info = client.instance_info().await.unwrap();
    assert_eq!(info.api_version, "v1");
    println!("instance: {} {}", info.name, info.version);

    if std::env::var("YAAMS_TOKEN").is_err() {
        return;
    }
    let user = client.current_user().await.unwrap();
    println!("user: {} ({} airlines)", user.name, user.airlines.len());
    let networks = client.online_networks().await.unwrap();
    println!("networks: {networks:?}");
    for airline in user.airlines.iter().filter(|a| a.is_member()) {
        let flights = client.flights(airline.id, &FlightQuery::default()).await.unwrap();
        let fleet = client.active_aircraft(airline.id).await.unwrap();
        println!(
            "{} {}: {} flights, {} active aircraft, abilities {:?}",
            airline.prefix,
            airline.name,
            flights.meta.total,
            fleet.len(),
            airline.abilities()
        );
    }
}
