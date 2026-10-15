use serde::Deserialize;

/// An online network a flight can be filed on. IDs differ per instance.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OnlineNetwork {
    pub id: u64,
    pub name: String,
}

impl std::fmt::Display for OnlineNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
