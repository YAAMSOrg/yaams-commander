//! API resource models. Field names follow the API's camelCase JSON.

mod aircraft;
mod airline;
mod flight;
mod instance;
mod network;
mod page;
mod user;

pub use aircraft::{Aircraft, AircraftQuery, AircraftStatus};
pub use airline::{Airline, AirlineAbilities, Membership};
pub use flight::{
    BLOCK_TIME_FORMAT, Flight, FlightQuery, FlightSort, FlightStatus, FlightStatusFilter, NewPirep, SortDirection,
};
pub use instance::{InstanceFeatures, InstanceInfo, LegalLinks};
pub use network::OnlineNetwork;
pub use page::{Page, PageLinks, PageMeta};
pub use user::{InstanceAbilities, User};

/// The `{ "data": ... }` envelope used by single-resource responses.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Data<T> {
    pub data: T,
}
