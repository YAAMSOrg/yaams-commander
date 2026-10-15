//! Typed client for the YAAMS REST API v1.
//!
//! All endpoints live under `<instance>/api/v1` and are authenticated with a
//! personal Sanctum bearer token, except [`Client::instance_info`].

mod client;
mod error;
pub mod models;
mod url;

pub use client::Client;
pub use error::{ApiError, Result};
pub use models::*;
pub use url::normalize_base_url;
