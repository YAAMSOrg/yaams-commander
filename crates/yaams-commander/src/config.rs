//! Persistent user configuration: connection profiles and UI preferences.
//!
//! Everything lives in a single TOML file in the platform's configuration
//! directory. The file holds API tokens, so it is written atomically and is
//! only readable by the current user on Unix.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yaams_api::ClientOptions;

/// Environment variable overriding the configuration file location.
pub const CONFIG_ENV: &str = "YAAMS_COMMANDER_CONFIG";
const FILE_NAME: &str = "config.toml";
const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "current_version")]
    pub version: u32,
    #[serde(default)]
    pub active_profile: Option<Uuid>,
    #[serde(default)]
    pub ui: UiPreferences,
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

fn current_version() -> u32 {
    CURRENT_VERSION
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            active_profile: None,
            ui: UiPreferences::default(),
            profiles: Vec::new(),
        }
    }
}

/// A saved connection to a YAAMS instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub token: String,
    /// Skip TLS certificate verification (self-signed certificates).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub accept_invalid_certs: bool,
    #[serde(default)]
    pub last_airline_id: Option<u64>,
    pub created_at: DateTime<Utc>,
}

impl Profile {
    pub fn new(name: String, base_url: String, token: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            base_url,
            token,
            accept_invalid_certs: false,
            last_airline_id: None,
            created_at: Utc::now(),
        }
    }

    /// Connection settings for the API client of this profile.
    pub fn client_options(&self) -> ClientOptions {
        ClientOptions {
            accept_invalid_certs: self.accept_invalid_certs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPreferences {
    /// Rows per page in lists (1-100).
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_per_page() -> u32 {
    25
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            per_page: default_per_page(),
        }
    }
}

impl Config {
    pub fn active(&self) -> Option<&Profile> {
        let id = self.active_profile?;
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn profile_mut(&mut self, id: Uuid) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|p| p.id == id)
    }

    /// Adds or replaces (same id) a profile and makes it active.
    pub fn upsert_profile(&mut self, profile: Profile) {
        self.active_profile = Some(profile.id);
        match self.profiles.iter_mut().find(|p| p.id == profile.id) {
            Some(existing) => *existing = profile,
            None => self.profiles.push(profile),
        }
    }

    /// Removes a profile; the first remaining one becomes active if needed.
    pub fn remove_profile(&mut self, id: Uuid) {
        self.profiles.retain(|p| p.id != id);
        if self.active_profile == Some(id) || self.active().is_none() {
            self.active_profile = self.profiles.first().map(|p| p.id);
        }
    }

    fn sanitize(&mut self) {
        self.ui.per_page = self.ui.per_page.clamp(1, 100);
        if self.active().is_none() {
            self.active_profile = self.profiles.first().map(|p| p.id);
        }
    }
}

/// Result of loading the configuration.
#[derive(Debug)]
pub struct Loaded {
    pub config: Config,
    /// Set when the existing file could not be read and was moved aside.
    pub notice: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    /// Store at the platform default location, or at `$YAAMS_COMMANDER_CONFIG`.
    pub fn from_env() -> Result<Self, String> {
        if let Some(path) = std::env::var_os(CONFIG_ENV).filter(|p| !p.is_empty()) {
            return Ok(Self::at(PathBuf::from(path)));
        }
        default_dir()
            .map(|dir| Self::at(dir.join(FILE_NAME)))
            .ok_or_else(|| "could not determine the configuration directory".to_string())
    }

    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads the configuration. A missing file yields the default; an
    /// unreadable one is backed up and replaced by the default.
    pub fn load(&self) -> Loaded {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Loaded {
                    config: Config::default(),
                    notice: None,
                };
            }
            Err(e) => {
                return Loaded {
                    config: Config::default(),
                    notice: Some(format!("Could not read {}: {e}", self.path.display())),
                };
            }
        };
        match toml::from_str::<Config>(&text) {
            Ok(mut config) => {
                config.sanitize();
                Loaded { config, notice: None }
            }
            Err(e) => {
                let backup = self
                    .path
                    .with_extension(format!("toml.bak-{}", Utc::now().format("%Y%m%d%H%M%S")));
                let moved = fs::rename(&self.path, &backup).is_ok();
                let notice = if moved {
                    format!(
                        "The configuration file was invalid and has been moved to {}. ({e})",
                        backup.display()
                    )
                } else {
                    format!("The configuration file is invalid: {e}")
                };
                Loaded {
                    config: Config::default(),
                    notice: Some(notice),
                }
            }
        }
    }

    /// Writes the configuration atomically (temp file + rename) with
    /// owner-only permissions on Unix.
    pub fn save(&self, config: &Config) -> Result<(), String> {
        let text = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        let dir = self.path.parent().ok_or("invalid configuration path")?;
        create_private_dir(dir).map_err(|e| format!("Could not create {}: {e}", dir.display()))?;

        let tmp = self.path.with_extension("toml.tmp");
        write_private(&tmp, text.as_bytes()).map_err(|e| format!("Could not write {}: {e}", tmp.display()))?;
        fs::rename(&tmp, &self.path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("Could not save {}: {e}", self.path.display())
        })
    }
}

/// Platform configuration directory of the app.
fn default_dir() -> Option<PathBuf> {
    let base = dirs::config_dir()?;
    let folder = if cfg!(any(target_os = "macos", target_os = "windows")) {
        "YAAMS Commander"
    } else {
        "yaams-commander"
    };
    Some(base.join(folder))
}

#[cfg(unix)]
fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    if dir.exists() {
        return Ok(());
    }
    fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("nested").join(FILE_NAME));
        (dir, store)
    }

    #[test]
    fn missing_file_yields_default() {
        let (_dir, store) = store();
        let loaded = store.load();
        assert_eq!(loaded.config, Config::default());
        assert!(loaded.notice.is_none());
    }

    #[test]
    fn round_trip_with_profiles() {
        let (_dir, store) = store();
        let mut config = Config::default();
        let a = Profile::new("Dev".into(), "https://dev.example/".into(), "tok-a".into());
        let mut b = Profile::new("Prod".into(), "https://prod.example/".into(), "tok-b".into());
        b.accept_invalid_certs = true;
        let a_id = a.id;
        config.upsert_profile(a);
        config.upsert_profile(b.clone());
        assert_eq!(config.active().unwrap().id, b.id);
        store.save(&config).unwrap();

        let loaded = store.load().config;
        assert_eq!(loaded, config);

        let mut loaded = loaded;
        loaded.remove_profile(b.id);
        assert_eq!(loaded.active().unwrap().id, a_id);
    }

    #[cfg(unix)]
    #[test]
    fn file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let (_dir, store) = store();
        store.save(&Config::default()).unwrap();
        let mode = fs::metadata(store.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let dir_mode = fs::metadata(store.path().parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dir_mode, 0o700);
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let (_dir, store) = store();
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        fs::write(store.path(), "this is = = not toml").unwrap();
        let loaded = store.load();
        assert_eq!(loaded.config, Config::default());
        assert!(loaded.notice.unwrap().contains("moved"));
        assert!(!store.path().exists());
    }

    #[test]
    fn sanitizes_values() {
        let (_dir, store) = store();
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        let id = Uuid::new_v4();
        fs::write(
            store.path(),
            format!("active_profile = \"{id}\"\n[ui]\nper_page = 1000\n"),
        )
        .unwrap();
        let config = store.load().config;
        assert_eq!(config.ui.per_page, 100);
        assert_eq!(config.active_profile, None);
    }
}
