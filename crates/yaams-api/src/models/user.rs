use serde::Deserialize;

use super::Airline;

/// The account an API token belongs to (`GET /api/v1/user`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub abilities: InstanceAbilities,
    #[serde(default)]
    pub airlines: Vec<Airline>,
}

/// Instance-level abilities. Missing or unknown keys mean `false`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAbilities {
    #[serde(default)]
    pub found_airline: bool,
    #[serde(default)]
    pub administer_instance: bool,
}
