use serde::Deserialize;

/// A page of a list endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub links: PageLinks,
    pub meta: PageMeta,
}

impl<T> Page<T> {
    pub fn has_next(&self) -> bool {
        self.meta.current_page < self.meta.last_page
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PageLinks {
    pub first: Option<String>,
    pub last: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PageMeta {
    pub current_page: u32,
    /// `None` for an empty page.
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub per_page: u32,
    pub total: u32,
    pub last_page: u32,
}
