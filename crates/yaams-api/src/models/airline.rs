use serde::Deserialize;

/// A virtual airline, including the token owner's membership.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Airline {
    pub id: u64,
    pub name: String,
    pub prefix: String,
    #[serde(default)]
    pub icao_callsign: String,
    #[serde(default)]
    pub atc_callsign: String,
    #[serde(default)]
    pub unit_is_lbs: bool,
    #[serde(default)]
    pub require_pirep_review: bool,
    #[serde(default)]
    pub location_continuity: bool,
    #[serde(default)]
    pub is_suspended: bool,
    /// `None` when the token owner is not a member.
    #[serde(default)]
    pub membership: Option<Membership>,
}

impl Airline {
    /// Abilities of the token owner in this airline. All `false` for
    /// non-members and suspended (read-only) airlines.
    pub fn abilities(&self) -> AirlineAbilities {
        match &self.membership {
            Some(m) if !self.is_suspended => m.abilities,
            _ => AirlineAbilities::default(),
        }
    }

    pub fn is_member(&self) -> bool {
        self.membership.is_some()
    }

    /// Fuel/weight unit label of the airline.
    pub fn weight_unit(&self) -> &'static str {
        if self.unit_is_lbs { "lbs" } else { "kg" }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    /// Display only - never gate UI on the role.
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub is_owner: bool,
    #[serde(default)]
    pub abilities: AirlineAbilities,
}

/// Per-airline abilities. Missing or unknown keys mean `false`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirlineAbilities {
    #[serde(default)]
    pub file_flights: bool,
    #[serde(default)]
    pub review_flights: bool,
    #[serde(default)]
    pub add_aircraft: bool,
    #[serde(default)]
    pub edit_aircraft: bool,
    #[serde(default)]
    pub manage_members: bool,
    #[serde(default)]
    pub manage_invite_codes: bool,
    #[serde(default)]
    pub manage_notams: bool,
    #[serde(default)]
    pub edit_settings: bool,
}
