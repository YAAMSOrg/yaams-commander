use serde::{Deserialize, Serialize};

/// An aircraft of an airline's fleet.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aircraft {
    pub id: u64,
    pub registration: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub current_loc: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub active: bool,
}

impl std::fmt::Display for Aircraft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  -  {} {}", self.registration, self.manufacturer, self.model)?;
        if let Some(loc) = &self.current_loc {
            write!(f, "  @ {loc}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AircraftStatus {
    Active,
    Inactive,
    Retired,
}

/// Filters for `GET /api/v1/airlines/{airline}/aircraft`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AircraftQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AircraftStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}
