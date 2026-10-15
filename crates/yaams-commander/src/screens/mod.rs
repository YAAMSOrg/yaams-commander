//! Application screens.

pub mod flights;
pub mod not_implemented;
pub mod pirep;
pub mod settings;
pub mod wizard;

use std::time::{Duration, Instant};

use uuid::Uuid;
use yaams_api::Flight;

use crate::widgets::toast::Kind;

/// Side effects a screen reports back to the application shell.
#[derive(Debug, Clone)]
pub enum Event {
    Toast(Kind, String, String),
    /// The token was rejected (401) - the profile needs a new one.
    Unauthorized,
    /// A request succeeded, with its round-trip time.
    Synced(Duration),
    PirepFiled(Box<Flight>),
    SaveConfig,
    OpenUrl(String),
    RevealConfig,
    SwitchProfile(Uuid),
    AddProfile,
    UpdateToken(Uuid),
    RemoveProfile(Uuid),
}

/// Runs a future and measures its duration.
pub async fn timed<T>(future: impl Future<Output = T>) -> (T, Duration) {
    let start = Instant::now();
    let value = future.await;
    (value, start.elapsed())
}
