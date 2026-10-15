use serde::Deserialize;

/// Public metadata of a YAAMS instance (`GET /api/v1/info`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub api_version: String,
    #[serde(default)]
    pub support_email: Option<String>,
    #[serde(default)]
    pub features: InstanceFeatures,
    #[serde(default)]
    pub legal: LegalLinks,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFeatures {
    #[serde(default)]
    pub registration: bool,
    #[serde(default)]
    pub user_airline_creation: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalLinks {
    #[serde(default)]
    pub imprint_url: Option<String>,
    #[serde(default)]
    pub privacy_url: Option<String>,
    #[serde(default)]
    pub terms_url: Option<String>,
}
