use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serialize};

/// Format of PIREP block times (UTC) in requests and responses.
pub const BLOCK_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// A flight (PIREP) of an airline.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flight {
    pub id: u64,
    #[serde(default)]
    pub callsign: String,
    /// Sent as a JSON integer by the server; strings are accepted as well.
    #[serde(default, deserialize_with = "string_or_integer")]
    pub flight_number: String,
    #[serde(default)]
    pub full_flight_number: String,
    #[serde(default)]
    pub full_icao_callsign: String,
    #[serde(default)]
    pub departure_icao: String,
    #[serde(default)]
    pub arrival_icao: String,
    #[serde(default)]
    pub cruise_altitude: Option<u32>,
    /// UTC, `Y-m-d H:i:s`.
    #[serde(default)]
    pub block_off: Option<String>,
    /// UTC, `Y-m-d H:i:s`.
    #[serde(default)]
    pub block_on: Option<String>,
    /// `HH:MM`.
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub burned_fuel: Option<u32>,
    #[serde(default)]
    pub route: Option<String>,
    /// Online network id, see [`OnlineNetwork`](super::OnlineNetwork).
    #[serde(default)]
    pub online_network: Option<u64>,
    pub status: FlightStatus,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(default)]
    pub rejection_remarks: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Flight {
    pub fn block_off_time(&self) -> Option<NaiveDateTime> {
        parse_block_time(self.block_off.as_deref()?)
    }

    pub fn block_on_time(&self) -> Option<NaiveDateTime> {
        parse_block_time(self.block_on.as_deref()?)
    }
}

fn string_or_integer<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        String(String),
        Integer(u64),
    }
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s,
        Value::Integer(n) => n.to_string(),
    })
}

fn parse_block_time(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, BLOCK_TIME_FORMAT).ok()
}

/// Workflow status of a flight.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FlightStatus {
    pub id: u8,
    pub name: String,
}

impl FlightStatus {
    pub const PENDING: u8 = 1;
    pub const ACCEPTED: u8 = 2;
    pub const REJECTED: u8 = 3;

    pub fn is_pending(&self) -> bool {
        self.id == Self::PENDING
    }

    pub fn is_accepted(&self) -> bool {
        self.id == Self::ACCEPTED
    }

    pub fn is_rejected(&self) -> bool {
        self.id == Self::REJECTED
    }
}

/// Status filter of the flight list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlightStatusFilter {
    #[default]
    All,
    Pending,
    Accepted,
    Rejected,
}

impl FlightStatusFilter {
    pub const ALL: [Self; 4] = [Self::All, Self::Pending, Self::Accepted, Self::Rejected];

    fn api_value(self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::Pending => Some("pending"),
            Self::Accepted => Some("accepted"),
            Self::Rejected => Some("rejected"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Pending => "Pending",
            Self::Accepted => "Accepted",
            Self::Rejected => "Rejected",
        }
    }
}

/// Documented sort keys of the flight list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightSort {
    FlightNumber,
    Callsign,
    Departure,
    Arrival,
    BlockOff,
    BlockOn,
    Duration,
    BurnedFuel,
    Status,
    CreatedAt,
}

impl FlightSort {
    pub fn api_key(self) -> &'static str {
        match self {
            Self::FlightNumber => "flight_number",
            Self::Callsign => "callsign",
            Self::Departure => "departure",
            Self::Arrival => "arrival",
            Self::BlockOff => "block_off",
            Self::BlockOn => "block_on",
            Self::Duration => "duration",
            Self::BurnedFuel => "burned_fuel",
            Self::Status => "status",
            Self::CreatedAt => "created_at",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

impl SortDirection {
    pub fn toggled(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Filters, sorting and paging for `GET /api/v1/airlines/{airline}/flights`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlightQuery {
    pub status: FlightStatusFilter,
    pub aircraft_id: Option<u64>,
    /// 4-letter ICAO code.
    pub departure: Option<String>,
    /// 4-letter ICAO code.
    pub arrival: Option<String>,
    /// Block-off on or after this UTC day.
    pub from: Option<NaiveDate>,
    /// Block-off on or before this UTC day.
    pub to: Option<NaiveDate>,
    pub sort: FlightSort,
    pub direction: SortDirection,
    pub page: u32,
    pub per_page: u32,
}

impl Default for FlightQuery {
    fn default() -> Self {
        Self {
            status: FlightStatusFilter::All,
            aircraft_id: None,
            departure: None,
            arrival: None,
            from: None,
            to: None,
            sort: FlightSort::BlockOff,
            direction: SortDirection::Descending,
            page: 1,
            per_page: 25,
        }
    }
}

impl FlightQuery {
    /// Query parameters as sent to the API.
    pub fn params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(status) = self.status.api_value() {
            params.push(("status", status.to_string()));
        }
        if let Some(id) = self.aircraft_id {
            params.push(("aircraft_id", id.to_string()));
        }
        if let Some(dep) = &self.departure {
            params.push(("departure", dep.clone()));
        }
        if let Some(arr) = &self.arrival {
            params.push(("arrival", arr.clone()));
        }
        if let Some(from) = self.from {
            params.push(("from", from.format("%Y-%m-%d").to_string()));
        }
        if let Some(to) = self.to {
            params.push(("to", to.format("%Y-%m-%d").to_string()));
        }
        let prefix = match self.direction {
            SortDirection::Ascending => "",
            SortDirection::Descending => "-",
        };
        params.push(("sort", format!("{prefix}{}", self.sort.api_key())));
        params.push(("page", self.page.max(1).to_string()));
        params.push(("per_page", self.per_page.clamp(1, 100).to_string()));
        params
    }
}

/// Request body of `POST /api/v1/airlines/{airline}/flights`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewPirep {
    pub flightnumber: u32,
    pub callsign: String,
    pub departure_icao: String,
    pub arrival_icao: String,
    pub aircraft_id: u64,
    pub crzalt: u32,
    #[serde(serialize_with = "serialize_block_time")]
    pub blockoff: NaiveDateTime,
    #[serde(serialize_with = "serialize_block_time")]
    pub blockon: NaiveDateTime,
    pub burned_fuel: u32,
    pub route: String,
    pub online_network_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

fn serialize_block_time<S: serde::Serializer>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&value.format(BLOCK_TIME_FORMAT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flight_number_accepts_integer_and_string() {
        for number in [serde_json::json!(331), serde_json::json!("331")] {
            let flight: Flight = serde_json::from_value(serde_json::json!({
                "id": 9, "flightNumber": number, "status": { "id": 2, "name": "Accepted" }
            }))
            .unwrap();
            assert_eq!(flight.flight_number, "331");
        }
    }

    #[test]
    fn default_query_params() {
        let params = FlightQuery::default().params();
        assert_eq!(
            params,
            vec![
                ("sort", "-block_off".to_string()),
                ("page", "1".to_string()),
                ("per_page", "25".to_string()),
            ]
        );
    }

    #[test]
    fn full_query_params() {
        let query = FlightQuery {
            status: FlightStatusFilter::Accepted,
            aircraft_id: Some(7),
            departure: Some("EDDF".into()),
            arrival: Some("EGLL".into()),
            from: NaiveDate::from_ymd_opt(2026, 7, 1),
            to: NaiveDate::from_ymd_opt(2026, 7, 31),
            sort: FlightSort::FlightNumber,
            direction: SortDirection::Ascending,
            page: 2,
            per_page: 500,
        };
        let params = query.params();
        assert!(params.contains(&("status", "accepted".into())));
        assert!(params.contains(&("from", "2026-07-01".into())));
        assert!(params.contains(&("sort", "flight_number".into())));
        assert!(params.contains(&("per_page", "100".into())));
    }

    #[test]
    fn pirep_serializes_block_times() {
        let pirep = NewPirep {
            flightnumber: 421,
            callsign: "421".into(),
            departure_icao: "EDDF".into(),
            arrival_icao: "EGLL".into(),
            aircraft_id: 7,
            crzalt: 36000,
            blockoff: NaiveDateTime::parse_from_str("2026-07-11 10:00:00", BLOCK_TIME_FORMAT).unwrap(),
            blockon: NaiveDateTime::parse_from_str("2026-07-11 11:30:00", BLOCK_TIME_FORMAT).unwrap(),
            burned_fuel: 4200,
            route: "SOVAT UL610 KONAN".into(),
            online_network_id: 1,
            remarks: None,
        };
        let json = serde_json::to_value(&pirep).unwrap();
        assert_eq!(json["blockoff"], "2026-07-11 10:00:00");
        assert_eq!(json["blockon"], "2026-07-11 11:30:00");
        assert!(json.get("remarks").is_none());
    }
}
